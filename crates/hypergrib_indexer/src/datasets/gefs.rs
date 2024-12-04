use anyhow::Context;
use hypergrib::{CoordLabels, GetCoordLabels};
use object_store::ObjectStore;
use std::sync::Arc;
use url::Url;

use crate::coord_labels_builder::CoordLabelsBuilder;

const BUCKET_URL: &str = "s3://noaa-gefs-pds";

pub struct Gefs {
    coord_labels_builder: CoordLabelsBuilder,
}

impl Gefs {
    pub fn new() -> anyhow::Result<Self> {
        let opts = vec![("skip_signature", "true")];
        let bucket_url = Url::try_from(BUCKET_URL)?;
        let (store, base_path) = object_store::parse_url_opts(&bucket_url, opts)?;
        let store: Arc<dyn ObjectStore> = Arc::from(store);
        let coord_labels_builder =
            CoordLabelsBuilder::new(store.clone(), base_path.clone(), store, base_path);
        Ok(Self {
            coord_labels_builder,
        })
    }
}

impl GetCoordLabels for Gefs {
    async fn get_coord_labels(self) -> anyhow::Result<CoordLabels> {
        // TODO: Append all coords to the coord_labels_builder!
        Ok(self.coord_labels_builder.build())
    }
}
