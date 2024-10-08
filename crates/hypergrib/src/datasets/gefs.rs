//! NOAA's Global Ensemble Forecast System (GEFS).
//! https://registry.opendata.aws/noaa-gefs

use std::{error::Error, fmt::Display, sync::Arc};

use chrono::{DateTime, NaiveDate, TimeDelta, Timelike, Utc};
use futures_util::StreamExt;
use object_store::ObjectStore;

use crate::filter_by_ext;

struct Gefs;

impl crate::ToIdxLocation for Gefs {
    fn to_idx_location(
        init_datetime: chrono::DateTime<chrono::Utc>,
        _product: String,
        _level: String,
        step: TimeDelta,
        ens_member: Option<u32>,
    ) -> object_store::path::Path {
        let mut parts = Vec::<object_store::path::PathPart>::with_capacity(3);
        let init_hour = format!("{:02}", init_datetime.hour());

        // First part of the Path:
        parts.push(init_datetime.format("gefs.%Y%m%d").to_string().into());

        // Second part of the Path:
        parts.push(init_hour.clone().into());

        // Third part of the Path:
        let ens_member = ens_member.unwrap();
        let ensemble_member = if ens_member == 0 {
            "gec00".to_string()
        } else {
            format!("gef{:02}", ens_member)
        };
        let forecast_step = if step == TimeDelta::zero() {
            "anl".to_string()
        } else {
            format!("f{:03}", step.num_hours())
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
        let mut list_stream = self
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
    path: object_store::path::Path,
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

// The path of the idx is like this:
// noaa-gefs-pds/
// gefs.<init date>/
// <init hour>/
// pgrb2b/
// gep<ensemble member>.t<init hour>z.pgrb2af<step>
struct GefsIdxLocation<'a> {
    path: &'a object_store::path::Path,
    parts: Vec<object_store::path::PathPart<'a>>,
}

impl<'a> TryFrom<&'a object_store::path::Path> for GefsIdxLocation<'a> {
    type Error = GefsIdxError;

    fn try_from(idx_location: &'a object_store::path::Path) -> Result<Self, Self::Error> {
        let parts: Vec<_> = idx_location.parts().collect();
        let i_start = match parts
            .iter()
            .position(|part| part.as_ref() == "noaa-gefs-pds")
        {
            Some(i) => i,
            None => {
                return Err(GefsIdxError {
                    error: format!("Failed to find 'noaa-gefs-pds'"),
                    path: idx_location.clone(),
                })
            }
        };
        let parts = &parts[i_start..];
        const N_PARTS_EXPECTED: usize = 5;
        if parts.len() != N_PARTS_EXPECTED {
            return Err(GefsIdxError {
                error: format!(
                    "Expected {N_PARTS_EXPECTED} parts in the path of the idx file. Found {} parts instead",
                    parts.len()
                    ),
                path: idx_location.clone(),
            });
        }
        Ok(Self {
            path: idx_location,
            parts: parts.to_vec(),
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
                error: format!("Invalid hour {hour} when parsing idx"),
                path: self.path.clone(),
            }
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::ToIdxLocation;

    use super::*;

    #[test]
    fn test_to_idx_location() -> anyhow::Result<()> {
        let p = Gefs::to_idx_location(
            // Note that the string on the line below includes minutes, even though the GEFS
            // idx files do not contain minutes. This is because `chrono::NaiveDateTime::parse_from_str`
            // throws an error if minutes aren't present in the string :(.
            NaiveDateTime::parse_from_str("201701010000", "%Y%m%d%H%M")
                .expect("parse datetime")
                .and_utc(),
            "HGT".to_string(),
            "10 mb".to_string(),
            TimeDelta::hours(6),
            Some(0),
        );
        assert_eq!(
            p,
            object_store::path::Path::from("gefs.20170101/00/gec00.t00z.pgrb2af006")
        );
        Ok(())
    }
}
