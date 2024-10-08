//! NOAA's Global Ensemble Forecast System (GEFS).
//! https://registry.opendata.aws/noaa-gefs

use std::sync::Arc;

use chrono::{TimeDelta, Timelike};
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
            self.coord_labels_builder
                .reference_datetime
                .insert(extract_reference_datetime(&meta.location));
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
