use std::sync::Arc;

pub mod datasets;
use chrono::{DateTime, TimeDelta, Utc};

// TODO: Replace this with Enums from gribberish.
#[derive(PartialEq, Eq, Hash, Clone)]
enum EnsembleMember {
    Control,
    Perturbed(u16),
    Mean,
    Spread,
}

// TODO: Key can probably be replaced by a similar enum in `hypergrib_idx_parser`?
// #[derive(PartialEq, Eq, Hash, Clone)] // PartialEq, Eq, and Hash are required for HashMap keys.
// struct Key {
//     reference_time: DateTime<Utc>,
//     ensemble_member: EnsembleMember, // Our own enum?
//     forecast_step: TimeDelta,
//     parameter: hypergrib_idx_parser::Parameter,
//     vertical_level: VerticalLevel, // From gribberish?
//     // Also for consideration:
//     // provider: Provider,  // e.g. NOAA, UKMetOffice, ECMWF, etc.
//     // nwp_model: NWPModel,  // e.g. GFS, GEFS, UKV, etc.
//     // or maybe combine `provider` and `nwp_model` into a single Enum e.g. UKMO_UKV, etc?
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

// TODO: Implement `struct CoordLabels` and `SortedVecSet<T>`:
// struct SortedVecSet<T>(Vec<T>);
//
// impl<T> SortedVecSet<T> {
//   /// Insert only if a duplicate doesn't exist. Sorts after insertion.
//   fn insert(t: T) -> Result<DuplicateExists>;
// }
//
// struct NwpCoordLabels {
//   // We're using `SortedVecSet` (not `BTreeSet`) because the most performance-sensitive
//   // part of the process is looking up a coord label given an integer index.
//   // And the only way to do that with a `BTreeSet` is to first iterate over the elements.
//   init_time: SortedVecSet<Datetime>,
//   ensemble_member: SortedVecSet<u16>,
//   forecast_step: SortedVecSet<Timedelta>,
//   nwp_variable: SortedVecSet<Variable>,
//   vertical_level: SortedVecSet<VerticalLevel>,
// }
//

trait ToIdxLocation {
    // TODO: Pass in a struct instead of individual fields?
    fn to_idx_location(
        init_datetime: DateTime<Utc>,
        product: String,
        level: String,
        step: TimeDelta,
        ens_member: Option<u32>,
    ) -> object_store::path::Path;
}
