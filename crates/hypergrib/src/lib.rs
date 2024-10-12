use std::{collections::HashSet, future, sync::Arc};

pub mod datasets;
use chrono::{DateTime, TimeDelta, Utc};
use futures_util::{Stream, StreamExt};
use object_store::{ObjectMeta, ObjectStore};

// #[derive(PartialEq, Eq, Hash, Clone)] // PartialEq, Eq, and Hash are required for HashMap keys.
// struct Key {
//     reference_datetime: DateTime<Utc>,
//     ensemble_member: String, // TODO: Convert to info from GDAL GRIB tables
//     forecast_step: TimeDelta,
//     parameter: String, //  TODO: Convert to info from GDAL GRIB tables
//     vertical_level: String, // TODO: Convert to info from GDAL GRIB tables
// }

/// The location of a GRIB message.
struct MessageLocation {
    path: Arc<object_store::path::Path>,
    byte_offset: u32,
    msg_length: u32,
    // TODO: Store a reference to coord labels for x and y?
    // TODO: Maybe a ref to a struct which holds lots of metadata about this grib message such as:
    // - coord labels for x and y
    // - NWP model version
    // - other metadata?
}

struct CoordLabelsBuilder {
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
    fn new(
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

    fn build(self) -> CoordLabels {
        CoordLabels {
            reference_datetime: set_to_sorted_vec(self.reference_datetime),
            ensemble_member: set_to_sorted_vec(self.ensemble_member),
            forecast_step: set_to_sorted_vec(self.forecast_step),
            parameter: set_to_sorted_vec(self.parameter),
            vertical_level: set_to_sorted_vec(self.vertical_level),
        }
    }
}

fn set_to_sorted_vec<T: Ord>(set: HashSet<T>) -> Vec<T> {
    let mut v: Vec<T> = set.into_iter().collect();
    v.sort();
    v
}

/// Each `vec` is sorted and contains unique values.
/// The only way to make a `CoordLabels` is using `CoordLabelsBuilder::build`.
struct CoordLabels {
    reference_datetime: Vec<DateTime<Utc>>,
    ensemble_member: Vec<String>,
    forecast_step: Vec<TimeDelta>,
    parameter: Vec<String>,
    vertical_level: Vec<String>,
}

/// Get the coordinate labels by reading parts of the GRIB dataset from object storage.
trait GetCoordLabels {
    async fn get_coord_labels(self) -> anyhow::Result<CoordLabels>;
}

trait ToIdxPath {
    // TODO: Pass in a struct instead of individual fields?
    fn to_idx_path(
        reference_datetime: &DateTime<Utc>,
        parameter: &str,
        vertical_level: &str,
        forecast_step: &TimeDelta,
        ensemble_member: Option<&str>,
    ) -> object_store::path::Path;
}

/// Filter a stream of `object_store::Result<object_store::ObjectMeta>` to select only the items
/// which have a file extension which matches `extension`.
pub fn filter_by_ext<'a>(
    stream: impl Stream<Item = object_store::Result<ObjectMeta>> + 'a,
    extension: &'static str,
) -> impl Stream<Item = object_store::Result<ObjectMeta>> + 'a {
    stream.filter(move |list_result| {
        future::ready(list_result.as_ref().is_ok_and(|meta| {
            meta.location
                .extension()
                .is_some_and(|ext| ext == extension)
        }))
    })
}
