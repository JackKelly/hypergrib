use std::{future, sync::Arc};

pub mod datasets;
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use futures_util::{Stream, StreamExt};
use object_store::ObjectMeta;

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

/// Each `Vec` must be sorted and contains unique values.
// TODO: Consider implementing a `SortedVec` struct which guarantees
// that elements are sorted and unique.
pub struct CoordLabels {
    pub reference_datetime: Vec<DateTime<Utc>>,
    pub ensemble_member: Vec<String>,
    pub forecast_step: Vec<TimeDelta>,
    pub parameter: Vec<String>,
    pub vertical_level: Vec<String>,
}

/// Get the coordinate labels.
pub trait GetCoordLabels {
    #[allow(async_fn_in_trait)]
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

pub(crate) fn ymdh_to_datetime(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
    match Utc.with_ymd_and_hms(year, month, day, hour, 0, 0) {
        chrono::offset::LocalResult::Single(dt) => dt,
        _ => panic!("Invalid datetime! {year}-{month}-{day}T{hour}"),
    }
}
