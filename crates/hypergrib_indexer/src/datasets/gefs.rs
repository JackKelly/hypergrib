use anyhow::Context;
use chrono::{DateTime, NaiveDate, Utc};
use hypergrib::{CoordLabels, GetCoordLabels};

use crate::coord_labels_builder::CoordLabelsBuilder;
use list_with_depth::list_with_depth;

const BUCKET_URL: &str = "s3://noaa-gefs-pds";
const SKIP_SIGNATURE: bool = true;

pub struct Gefs {
    coord_labels_builder: CoordLabelsBuilder,
}

impl Gefs {
    pub fn new() -> anyhow::Result<Self> {
        let coord_labels_builder = CoordLabelsBuilder::new_from_url(BUCKET_URL, SKIP_SIGNATURE)?;
        Ok(Self {
            coord_labels_builder,
        })
    }

    /// The reference datetimes are extracted from the first two parts of the path, for example:
    /// `gefs.20241204/00/`.
    async fn get_reference_datetimes(&mut self) -> anyhow::Result<()> {
        let store = self.coord_labels_builder.idx_store().clone();
        let prefix = self.coord_labels_builder.idx_base_path();
        let list = list_with_depth(store, Some(prefix), 1).await?;

        for prefix in list.common_prefixes.iter() {
            let datetime = path_to_reference_datetime(prefix)?;
            let datetime_is_unique = self
                .coord_labels_builder
                .insert_reference_datetime(datetime);
            assert!(
                datetime_is_unique,
                "Duplicate reference datetime! {datetime}"
            );
        }

        Ok(())
    }
}

impl GetCoordLabels for Gefs {
    async fn get_coord_labels(mut self) -> anyhow::Result<CoordLabels> {
        self.get_reference_datetimes().await?;
        // TODO: Append all coords to the coord_labels_builder!
        Ok(self.coord_labels_builder.build())
    }
}

/// Convert the first two parts of a path to a reference datetime.
/// For example, `gefs.20191122/18` becomes 2019-11-22T18:00.
fn path_to_reference_datetime(path: &object_store::path::Path) -> anyhow::Result<DateTime<Utc>> {
    let parts: Vec<_> = path.parts().take(2).collect();
    let error_context = |s| format!("{s} when parsing path: '{path}'");
    let date = NaiveDate::parse_from_str(parts[0].as_ref(), "gefs.%Y%m%d").with_context(|| {
        error_context("Failed to convert date component of NWP reference datetime")
    })?;
    let hour: u32 = parts[1]
        .as_ref()
        .parse()
        .with_context(|| error_context("Hour of the NWP init could not be parsed into a u32"))?;
    match date.and_hms_opt(hour, 0, 0) {
        Some(dt) => Ok(dt.and_utc()),
        None => Err(anyhow::format_err!(error_context(
            "Invalid NWP reference hour"
        ))),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_path_to_reference_datetime() -> anyhow::Result<()> {
        let path = object_store::path::Path::from("gefs.20191122/18");
        let result = path_to_reference_datetime(&path)?;
        assert_eq!(
            result,
            DateTime::parse_from_rfc3339("2019-11-22T18:00:00Z")?
        );
        Ok(())
    }
}
