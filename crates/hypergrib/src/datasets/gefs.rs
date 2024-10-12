//! NOAA's Global Ensemble Forecast System (GEFS).
//! https://registry.opendata.aws/noaa-gefs

use std::{error::Error, fmt::Display, sync::Arc};

use chrono::{DateTime, NaiveDate, TimeDelta, Timelike, Utc};
use futures_util::StreamExt;
use object_store::ObjectStore;

use crate::filter_by_ext;

const BUCKET_ID: &str = "noaa-gefs-pds";
struct Gefs;

impl crate::ToIdxLocation for Gefs {
    fn to_idx_location(
        reference_datetime: &chrono::DateTime<chrono::Utc>,
        _parameter: &str,
        _vertical_level: &str,
        forecast_step: &TimeDelta,
        ensemble_member: Option<&str>,
    ) -> object_store::path::Path {
        let mut parts = Vec::<object_store::path::PathPart>::with_capacity(3);
        let init_hour = format!("{:02}", reference_datetime.hour());

        // First part of the Path:
        parts.push(reference_datetime.format("gefs.%Y%m%d").to_string().into());

        // Second part of the Path:
        parts.push(init_hour.clone().into());

        // Third part of the Path:
        let ensemble_member = ensemble_member.expect("GEFS requires the ensemble member!");
        let forecast_step = if *forecast_step == TimeDelta::zero() {
            "anl".to_string()
        } else {
            format!("f{:03}", forecast_step.num_hours())
        };
        parts.push(
            format!(
                "{ensemble_member}.t{init_hour}z.pgrb2a{forecast_step}",
                ensemble_member = ensemble_member,
                init_hour = init_hour,
                forecast_step = forecast_step,
            )
            .into(),
        );
        object_store::path::Path::from_iter(parts)
    }
}

struct GefsCoordLabelsBuilder {
    coord_labels_builder: crate::CoordLabelsBuilder,
}

impl GefsCoordLabelsBuilder {
    fn new(
        grib_store: Arc<dyn ObjectStore>,
        grib_base_path: object_store::path::Path,
        idx_store: Arc<dyn ObjectStore>,
        idx_base_path: object_store::path::Path,
    ) -> Self {
        Self {
            coord_labels_builder: crate::CoordLabelsBuilder::new(
                grib_store,
                grib_base_path,
                idx_store,
                idx_base_path,
            ),
        }
    }

    async fn extract_from_idx_paths(&mut self) -> anyhow::Result<()> {
        // Get an `async` stream of Metadata objects:
        let list_stream = self
            .coord_labels_builder
            .idx_store
            .list(Some(&self.coord_labels_builder.idx_base_path));

        let mut list_stream = filter_by_ext(list_stream, "idx");

        // Loop through each .idx filename:
        while let Some(meta) = list_stream.next().await.transpose().unwrap() {
            let gefs_idx_loc = GefsIdxLocation::try_from(&meta.location)?;
            self.coord_labels_builder
                .reference_datetime
                .insert(gefs_idx_loc.reference_datetime()?);
        }
        Ok(())
    }
}

impl crate::GetCoordLabels for GefsCoordLabelsBuilder {
    async fn get_coord_labels(mut self) -> anyhow::Result<crate::CoordLabels> {
        self.extract_from_idx_paths().await?;
        // TODO: self.extract_from_gribs().await?; // Maybe do concurrently with
        // extract_from_idx_paths?
        Ok(self.coord_labels_builder.build())
    }
}

#[derive(Debug)]
struct GefsIdxError {
    path: String,
    error: String,
}

impl Display for GefsIdxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {}. For GEFS .idx path: {}",
            self.error, self.path
        )
    }
}

impl Error for GefsIdxError {}

// The location of the `.idx` file is structured like this:
//     noaa-gefs-pds/
//     gefs.<YYYYMMDD>/
//     <HH>/
//     <atmos | chem | wave>/  # Only in newer forecasts
//     <bufr | init | pgrb2ap5 | pgrb2bp5 | pgrb2sp25>/  # Only in newer forecasts
//     gep<ensemble member>.t<HH>z.pgrb2af<step>
// For example:
// - `noaa-gefs-pds/gefs.20170101/00/`
//     - `gec00.t00z.pgrb2aanl.idx`
//     - `gec00.t00z.pgrb2bf330.idx`
//     - `gep20.t00z.pgrb2bf384.idx`
// - `noaa-gefs-pds/gefs.20241010/00/atmos/pgrb2ap5/`
//     - `geavg.t00z.pgrb2a.0p50.f000.idx`
//     - `geavg.t00z.pgrb2a.0p50.f840.idx`
//     - `gespr.t00z.pgrb2a.0p50.f840.idx`
//     - `gec00.t00z.pgrb2a.0p50.f000.idx`
struct GefsIdxLocation<'a> {
    path: &'a object_store::path::Path,
    parts: Vec<object_store::path::PathPart<'a>>,
}

impl<'a> TryFrom<&'a object_store::path::Path> for GefsIdxLocation<'a> {
    type Error = GefsIdxError;

    fn try_from(idx_location: &'a object_store::path::Path) -> Result<Self, Self::Error> {
        let parts: Vec<_> = idx_location.parts().collect();

        // Helper closure:
        let make_error = |error: String| -> Result<Self, Self::Error> {
            Err(GefsIdxError {
                error,
                path: idx_location.to_string(),
            })
        };

        // Sanity checks:
        if parts[0].as_ref() != BUCKET_ID {
            return make_error(format!("GEFS location must start with '{BUCKET_ID}'."));
        }

        // Check the number of parts:
        let n_parts = parts.len();
        const N_PARTS_EXPECTED: [usize; 2] = [4, 6];
        if !N_PARTS_EXPECTED.contains(&n_parts) {
            return make_error(format!(
                "Expected {N_PARTS_EXPECTED:?} parts in the location of the idx file. Found {n_parts} parts instead.",
            ));
        }

        // Check that the path ends with `.idx`.
        let last_part = &parts[n_parts - 1];
        if !last_part.as_ref().ends_with(".idx") {
            return make_error("The location must end with '.idx'!".to_string());
        }

        Ok(Self {
            path: idx_location,
            parts,
        })
    }
}

impl GefsIdxLocation<'_> {
    fn reference_datetime(&self) -> anyhow::Result<DateTime<Utc>> {
        let date = NaiveDate::parse_from_str(self.parts[1].as_ref(), "gefs.%Y%m%d")?;
        let hour: u32 = self.parts[2].as_ref().parse()?;
        match date.and_hms_opt(hour, 0, 0) {
            Some(dt) => Ok(dt.and_utc()),
            None => Err(GefsIdxError {
                error: format!("Invalid hour {hour} when parsing location of idx"),
                path: self.path.to_string(),
            }
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime, TimeZone};

    use crate::ToIdxLocation;

    use super::*;

    const IDX_TEST_PATHS: [&str; 7] = [
        "noaa-gefs-pds/gefs.20170101/00/gec00.t00z.pgrb2aanl.idx",
        "noaa-gefs-pds/gefs.20170101/00/gec00.t00z.pgrb2bf330.idx",
        "noaa-gefs-pds/gefs.20170101/00/gep20.t00z.pgrb2bf384.idx",
        "noaa-gefs-pds/gefs.20241010/00/atmos/pgrb2ap5/geavg.t00z.pgrb2a.0p50.f000.idx",
        "noaa-gefs-pds/gefs.20241010/00/atmos/pgrb2ap5/geavg.t00z.pgrb2a.0p50.f840.idx",
        "noaa-gefs-pds/gefs.20241010/00/atmos/pgrb2ap5/gespr.t00z.pgrb2a.0p50.f840.idx",
        "noaa-gefs-pds/gefs.20241010/00/atmos/pgrb2ap5/gec00.t00z.pgrb2a.0p50.f000.idx",
    ];

    #[test]
    fn test_idx_loc_to_reference_datetime() {
        [
            Utc.with_ymd_and_hms(2017, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2017, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2017, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 10, 10, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 10, 10, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 10, 10, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 10, 10, 0, 0, 0).unwrap(),
        ]
        .into_iter()
        .zip(IDX_TEST_PATHS.iter())
        .for_each(|(expected_datetime, path_str)| {
            let path = object_store::path::Path::try_from(*path_str).expect(&format!(
                "Failed to parse path string into an object_store::path::Path! {path_str}"
            ));
            let dt = GefsIdxLocation::try_from(&path)
                .unwrap()
                .reference_datetime()
                .expect(&format!(
                    "Failed to extract reference datetime from {path_str}"
                ));
            assert_eq!(
                dt, expected_datetime,
                "Incorrect reference datetime when parsing idx location '{path}'"
            );
        });
    }

    #[test]
    fn test_to_idx_location() -> anyhow::Result<()> {
        let p = Gefs::to_idx_location(
            // Note that the string on the line below includes minutes, even though the GEFS
            // idx files do not contain minutes. This is because `chrono::NaiveDateTime::parse_from_str`
            // throws an error if minutes aren't present in the string :(.
            &NaiveDateTime::parse_from_str("201701010000", "%Y%m%d%H%M")
                .expect("parse datetime")
                .and_utc(),
            "HGT",
            "10 mb",
            &TimeDelta::hours(6),
            Some("gec00"),
        );
        assert_eq!(
            p,
            object_store::path::Path::from("gefs.20170101/00/gec00.t00z.pgrb2af006")
        );
        Ok(())
    }
}
