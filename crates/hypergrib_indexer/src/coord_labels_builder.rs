use std::{collections::BTreeSet, sync::Arc};

use chrono::{DateTime, TimeDelta, Utc};
use hypergrib::CoordLabels;
use object_store::{limit::LimitStore, ObjectStore};
use url::Url;

/// Set the maximum number of concurrent operations.
/// Set to `None` for no limit. But beware that, when no limit is set,
/// you may have to increase the number of open file descriptors that your operating
/// system allows. (Note that file descriptors are used for network connections, as
/// well as for files!). On Linux, run `ulimit -n 10000` to set the limit to 10,000.
const CONCURRENCY_LIMIT: Option<usize> = Some(1000);

pub(crate) struct CoordLabelsBuilder {
    grib_store: Arc<dyn ObjectStore>,
    grib_base_path: object_store::path::Path,
    idx_store: Arc<dyn ObjectStore>,
    idx_base_path: object_store::path::Path,
    reference_datetime: BTreeSet<DateTime<Utc>>,
    ensemble_member: BTreeSet<String>,
    forecast_step: BTreeSet<TimeDelta>,
    parameter: BTreeSet<String>,
    vertical_level: BTreeSet<String>,
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
            reference_datetime: BTreeSet::new(),
            ensemble_member: BTreeSet::new(),
            forecast_step: BTreeSet::new(),
            parameter: BTreeSet::new(),
            vertical_level: BTreeSet::new(),
        }
    }

    pub(crate) fn new_from_url(url: &str, skip_signature: bool) -> anyhow::Result<Self> {
        let mut opts = vec![];
        if skip_signature {
            opts.push(("skip_signature", "true"));
        }
        let bucket_url = Url::try_from(url)?;
        let (store, base_path) = object_store::parse_url_opts(&bucket_url, opts)?;
        let store: Arc<dyn ObjectStore> = if let Some(concurrency_limit) = CONCURRENCY_LIMIT {
            Arc::new(LimitStore::new(store, concurrency_limit))
        } else {
            Arc::from(store)
        };
        Ok(CoordLabelsBuilder::new(
            store.clone(),
            base_path.clone(),
            store,
            base_path,
        ))
    }

    pub(crate) fn build(self) -> CoordLabels {
        CoordLabels {
            reference_datetime: to_sorted_vec(self.reference_datetime),
            ensemble_member: to_sorted_vec(self.ensemble_member),
            forecast_step: to_sorted_vec(self.forecast_step),
            parameter: to_sorted_vec(self.parameter),
            vertical_level: to_sorted_vec(self.vertical_level),
        }
    }

    pub(crate) fn grib_store(&self) -> &Arc<dyn ObjectStore> {
        &self.grib_store
    }

    pub(crate) fn grib_base_path(&self) -> &object_store::path::Path {
        &self.grib_base_path
    }

    pub(crate) fn idx_store(&self) -> &Arc<dyn ObjectStore> {
        &self.idx_store
    }

    pub(crate) fn idx_base_path(&self) -> &object_store::path::Path {
        &self.idx_base_path
    }

    pub(crate) fn insert_reference_datetime(&mut self, datetime: DateTime<Utc>) -> bool {
        self.reference_datetime.insert(datetime)
    }

    pub(crate) fn reference_datetime(&self) -> &BTreeSet<DateTime<Utc>> {
        &self.reference_datetime
    }

    pub(crate) fn describe_reference_datetimes(&self) -> String {
        let dts = &self.reference_datetime;
        format!(
            "{} reference datetimes found. First: {:?}. Last: {:?}",
            dts.len(),
            dts.first(),
            dts.last()
        )
    }
}

fn to_sorted_vec<T, S>(set: S) -> Vec<T>
where
    T: Ord,
    S: IntoIterator,
    Vec<T>: FromIterator<<S as IntoIterator>::Item>,
{
    let mut v: Vec<T> = set.into_iter().collect();
    v.sort();
    v
}
