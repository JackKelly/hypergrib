use hypergrib::{CoordLabels, GetCoordLabels};

use crate::coord_labels_builder::CoordLabelsBuilder;

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

    /// The reference datetimes in extracted from the first two parts of the path, for example:
    /// `gefs.20241204/00/`.
    async fn get_reference_datetimes(&self) -> anyhow::Result<()> {
        let list = self
            .coord_labels_builder
            .grib_store()
            .list_with_delimiter(None)
            .await?;

        let n = list.common_prefixes.len();
        println!(
            "Number of ListResult.common_prefixes: {n}, the last is {:?}",
            list.common_prefixes[n - 1]
        );

        // TODO:
        // - For each directory like `gefs.20241204`, find all the init hours by calling
        //   list_with_delimiter again. In parallel.
        // - Convert to `DateTime<Utc>` using code like this:
        //   https://github.com/JackKelly/hypergrib/issues/22#issuecomment-2517163383
        // - Insert these into `self.coord_labels_builder.reference_datetime` HashSet.
        //   HashSet<T> is Send and Sync if T is Send and Sync.

        Ok(())
    }
}

impl GetCoordLabels for Gefs {
    async fn get_coord_labels(self) -> anyhow::Result<CoordLabels> {
        self.get_reference_datetimes().await?;
        // TODO: Append all coords to the coord_labels_builder!
        Ok(self.coord_labels_builder.build())
    }
}
