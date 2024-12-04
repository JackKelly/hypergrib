use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, TimeDelta, Utc};
use hypergrib::CoordLabels;
use object_store::ObjectStore;
use url::Url;

pub(crate) struct CoordLabelsBuilder {
    grib_store: Arc<dyn ObjectStore>,
    grib_base_path: object_store::path::Path,
    idx_store: Arc<dyn ObjectStore>,
    idx_base_path: object_store::path::Path,
    reference_datetime: HashSet<DateTime<Utc>>,
    ensemble_member: HashSet<String>,
    forecast_step: HashSet<TimeDelta>,
    parameter: HashSet<String>,
    vertical_level: HashSet<String>,
}

impl CoordLabelsBuilder {
    pub(crate) fn new(
        grib_store: Arc<dyn ObjectStore>,
        grib_base_path: object_store::path::Path,
        idx_store: Arc<dyn ObjectStore>,
        idx_base_path: object_store::path::Path,
    ) -> Self {
        Self {
            grib_store,
            grib_base_path,
            idx_store,
            idx_base_path,
            reference_datetime: HashSet::new(),
            ensemble_member: HashSet::new(),
            forecast_step: HashSet::new(),
            parameter: HashSet::new(),
            vertical_level: HashSet::new(),
        }
    }

    pub(crate) fn new_from_url(url: &str, skip_signature: bool) -> anyhow::Result<Self> {
        let mut opts = vec![];
        if skip_signature {
            opts.push(("skip_signature", "true"));
        }
        let bucket_url = Url::try_from(url)?;
        let (store, base_path) = object_store::parse_url_opts(&bucket_url, opts)?;
        let store: Arc<dyn ObjectStore> = Arc::from(store);
        Ok(CoordLabelsBuilder::new(
            store.clone(),
            base_path.clone(),
            store,
            base_path,
        ))
    }

    pub(crate) fn build(self) -> CoordLabels {
        CoordLabels {
            reference_datetime: set_to_sorted_vec(self.reference_datetime),
            ensemble_member: set_to_sorted_vec(self.ensemble_member),
            forecast_step: set_to_sorted_vec(self.forecast_step),
            parameter: set_to_sorted_vec(self.parameter),
            vertical_level: set_to_sorted_vec(self.vertical_level),
        }
    }

    pub(crate) fn grib_store(&self) -> &dyn ObjectStore {
        &self.grib_store
    }

    pub(crate) fn grib_base_path(&self) -> &str {
        self.grib_base_path.as_ref()
    }

    pub(crate) fn idx_store(&self) -> &dyn ObjectStore {
        &self.idx_store
    }

    pub(crate) fn idx_base_path(&self) -> &str {
        self.idx_base_path.as_ref()
    }
}

fn set_to_sorted_vec<T: Ord>(set: HashSet<T>) -> Vec<T> {
    let mut v: Vec<T> = set.into_iter().collect();
    v.sort();
    v
}
