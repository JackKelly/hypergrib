//! NOAA's Global Ensemble Forecast System (GEFS).
//! https://registry.opendata.aws/noaa-gefs

use std::{error::Error, fmt::Display, sync::Arc};

use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone, Timelike, Utc};
use futures_util::StreamExt;
use object_store::ObjectStore;

use crate::filter_by_ext;

const BUCKET_ID: &str = "noaa-gefs-pds";
struct Gefs;

impl crate::ToIdxPath for Gefs {
    fn to_idx_path(
        reference_datetime: &chrono::DateTime<chrono::Utc>,
        _parameter: &str,
        _vertical_level: &str,
        forecast_step: &TimeDelta,
        ensemble_member: Option<&str>,
    ) -> object_store::path::Path {
        // TODO: The code below only works for "old" GEFS paths. But GEFS switched to new,
        // more complicated paths at some point. For the hypergrib MVP, change this function to
        // only handle the new paths. Then implement both "old" and "new" and switch on the
        // `reference_datetime` to decide whether to use "old" or "new" format. And, for the "new"
        // format, have a `phf::Map` which tells us whether the parameter belongs to 'atmos',
        // 'chem', 'wave'; and 'pgrb2a' or 'pgrb2b' etc.
        let mut parts = Vec::<object_store::path::PathPart>::with_capacity(3);

        // First part of the Path:
        parts.push(reference_datetime.format("gefs.%Y%m%d").to_string().into());

        // Second part of the Path:
        let init_hour = format!("{:02}", reference_datetime.hour());
        parts.push(init_hour.as_str().into());

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
            let gefs_idx_path = GefsIdxPath::try_from(&meta.location)?;
            self.coord_labels_builder
                .reference_datetime
                .insert(gefs_idx_path.reference_datetime()?);
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

#[allow(non_camel_case_types)]
enum GefsIdxPathVersion {
    /// GEFS model version 11?
    ///
    /// Paths of the form `gefs.20170101/00/gec00.t00z.pgrb2aanl.idx`
    V1,

    /// GEFS model version 11?
    ///
    /// Paths of the form `gefs.20180727/00/pgrb2[a|b]/gec00.t00z.pgrb2aanl.idx`
    V2,

    /// A union of the paths in the previous and subsequent versions! i.e. contains
    /// paths of the form `gefs.20180727/00/pgrb2[a|b]/gec00.t00z.pgrb2aanl.idx` and
    /// paths of the form `gefs.20200923/00/[atmos|chem|wave]/etc...`
    V3,

    /// GEFS model version v12?
    ///
    /// Paths of the forms:
    ///              This character-----v---v---v is repeated here-----v-v-v
    ///              These characters----vv--vv--vvv are repeated here---------v-vv-v
    /// - `gefs.20241008/00/atmos/pgrb2[ap5|bp5|sp25]/geavg.t00z.pgrb2[a|b|s].0p[25|50].f000.idx`
    /// - `gefs.20241008/00/atmos/[bufr|init]/`: Ignore! No GRIB data.
    /// - `gefs.20241008/00/chem/[pgrb2ap25|pgrb2ap5]/gefs.chem.t00z.a2d_0p25.f000.grib2.idx`
    /// - `gefs.20241008/00/wave/`
    ///     - `gridded/gefs.wave.t00z.c00.global.0p25.f000.grib2.idx`
    ///     - `station`: Ignore! No GRIB data!
    V4,
}

impl GefsIdxPathVersion {
    fn start_datetime(&self) -> DateTime<Utc> {
        match *self {
            Self::V1 => Utc.with_ymd_and_hms(2017, 1, 1, 0, 0, 0),
            Self::V2 => Utc.with_ymd_and_hms(2018, 7, 27, 0, 0, 0),
            Self::V3 => Utc.with_ymd_and_hms(2020, 9, 23, 0, 0, 0),
            Self::V4 => Utc.with_ymd_and_hms(2020, 9, 23, 12, 0, 0),
        }
        .unwrap()
    }

    fn end_datetime(&self) -> DateTime<Utc> {
        match *self {
            Self::V1 => Utc.with_ymd_and_hms(2018, 7, 26, 18, 0, 0).unwrap(),
            Self::V2 => Utc.with_ymd_and_hms(2020, 9, 22, 18, 0, 0).unwrap(),
            Self::V3 => Utc.with_ymd_and_hms(2020, 9, 23, 6, 0, 0).unwrap(),
            Self::V4 => <DateTime<Utc>>::MAX_UTC,
        }
    }
}

// The path of the `.idx` file is structured like this:
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
struct GefsIdxPath<'a> {
    path: &'a object_store::path::Path,
    parts: Vec<object_store::path::PathPart<'a>>,
}

impl<'a> TryFrom<&'a object_store::path::Path> for GefsIdxPath<'a> {
    type Error = GefsIdxError;

    fn try_from(idx_path: &'a object_store::path::Path) -> Result<Self, Self::Error> {
        let parts: Vec<_> = idx_path.parts().collect();

        // Helper closure:
        let make_error = |error: String| -> Result<Self, Self::Error> {
            Err(GefsIdxError {
                error,
                path: idx_path.to_string(),
            })
        };

        // Sanity checks:
        if parts[0].as_ref() != BUCKET_ID {
            return make_error(format!("GEFS path must start with '{BUCKET_ID}'."));
        }

        // Check the number of parts:
        let n_parts = parts.len();
        const N_PARTS_EXPECTED: [usize; 2] = [4, 6];
        if !N_PARTS_EXPECTED.contains(&n_parts) {
            return make_error(format!(
                "Expected {N_PARTS_EXPECTED:?} parts in the path of the idx file. Found {n_parts} parts instead.",
            ));
        }

        // Check that the path ends with `.idx`.
        let last_part = &parts[n_parts - 1];
        if !last_part.as_ref().ends_with(".idx") {
            return make_error("The path must end with '.idx'!".to_string());
        }

        Ok(Self {
            path: idx_path,
            parts,
        })
    }
}

impl GefsIdxPath<'_> {
    fn reference_datetime(&self) -> anyhow::Result<DateTime<Utc>> {
        let date = NaiveDate::parse_from_str(self.parts[1].as_ref(), "gefs.%Y%m%d")?;
        let hour: u32 = self.parts[2].as_ref().parse()?;
        match date.and_hms_opt(hour, 0, 0) {
            Some(dt) => Ok(dt.and_utc()),
            None => Err(GefsIdxError {
                error: format!("Invalid hour {hour} when parsing path of idx"),
                path: self.path.to_string(),
            }
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDateTime, TimeZone};

    use crate::ToIdxPath;

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
    fn test_idx_path_to_reference_datetime() {
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
            let dt = GefsIdxPath::try_from(&path)
                .unwrap()
                .reference_datetime()
                .expect(&format!(
                    "Failed to extract reference datetime from {path_str}"
                ));
            assert_eq!(
                dt, expected_datetime,
                "Incorrect reference datetime when parsing idx path '{path}'"
            );
        });
    }

    #[test]
    fn test_to_idx_path() -> anyhow::Result<()> {
        let p = Gefs::to_idx_path(
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
