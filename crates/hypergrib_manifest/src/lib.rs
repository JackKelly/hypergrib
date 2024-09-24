#![doc = include_str!("../README.md")]

pub mod datasets;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, TimeDelta, Utc};
use gribberish::templates::product::parameters::meteorological;

// TODO: Replace this with Enums from gribberish.
#[derive(PartialEq, Eq, Hash)]
enum EnsembleMember {
    Control,
    Perturbed(u16),
    Mean,
    Spread,
}

// TODO: Replace this with Enums from gribberish. See https://github.com/mpiannucci/gribberish/issues/59
// TODO: Include all parameters listed for GEFS here:
// - https://www.nco.ncep.noaa.gov/pmb/products/gens/gep01.t00z.pgrb2a.0p50.f003.shtml
// - https://www.nco.ncep.noaa.gov/pmb/products/gens
/// Adapted from https://www.nco.ncep.noaa.gov/pmb/products/gens/gec00.t00z.pgrb2a.0p50.f000.shtml
#[derive(PartialEq, Eq, Hash)] // PartialEq, Eq, and Hash are required for HashMap keys.
enum Parameter {
    // The unit is after the underscore
    GeopotentialHeight_gpm,
    Temperature_K,
    RelativeHumidity_percent,
    UComponentOfWind_meters_per_sec,
    VComponentOfWind_meters_per_sec,
    VerticalVelocityAKAPressure_Pa_per_sec,
}

// TODO: Replace this with Enums from gribberish. See https://github.com/mpiannucci/gribberish/issues/59
/// Adapted from https://www.nco.ncep.noaa.gov/pmb/products/gens
#[derive(PartialEq, Eq, Hash)] // PartialEq, Eq, and Hash are required for HashMap keys.
enum VerticalLevel {
    Mb10,
    Mb50,
    Mb100,
    Mb200,
    Mb250,
    Mb300,
    Mb400,
    Mb500,
    Mb700,
    Mb850,
    Mb925,
    Mb1000,
    Surface,
    OneCentimeterBelowGround,
    TwoMetersAboveGround,
    TenMetersAboveGround,
    EntireAtmosphere,
    OneHundredAndEightyMbAboveGround,
    MeanSeaLevel,
    TopOfAtmosphere,
}

#[derive(PartialEq, Eq, Hash)] // PartialEq, Eq, and Hash are required for HashMap keys.
struct Key {
    reference_time: DateTime<Utc>,
    ensemble_member: EnsembleMember,
    forecast_step: TimeDelta,
    parameter: Parameter, // `Variable` is an enum
    vertical_level: VerticalLevel, // `VerticalLevel` is an enum?
                          // Also for consideration:
                          // provider: Provider,  // e.g. NOAA, UKMetOffice, ECMWF, etc.
                          // nwp_model: NWPModel,  // e.g. GFS, GEFS, UKV, etc.
                          // or maybe combine `provider` and `nwp_model` into a single Enum e.g. UKMO_UKV, etc?
}

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

struct Manifest {
    // TODO: Add coord_labels: CoordLabels,
    // Store the paths once, so we only have one Arc per Path.
    // Each path in `paths` will be relative to `base_path`.
    base_path: object_store::path::Path,
    paths: HashSet<Arc<object_store::path::Path>>,
    manifest: HashMap<Key, MessageLocation>,
    // Maybe we also want a `manifest_index` which maps integer indexes to `MessageLocation`
    // but let's make a start with the design below and benchmark it.
}

impl Manifest {
    fn insert(&mut self, key: Key, chunk: MessageLocation) -> Result<(), AlreadyExistsError> {
        // Insert into `manifest` and update `coord_labels` iff the new coord doesn't exist yet.
    }

    fn coord_labels_to_chunk(&self, key: &Key) -> Option<MessageLocation> {
        self.manifest[key]
    }

    fn index_locs_to_key(&self, index: &[u64]) -> Option<Key> {
        // get key by looking up the appropriate coord labels in self.coord_labels.
        // Returns `None` if any index is out of bounds (which is the same semantics as `Vec::get`).
        // Although maybe it'd be better to return a custom `Error` so we can say which dim
        // is out of bounds? Or if there are the wrong number of dims in the `index`?
        Some(key)
    }
}

trait Dataset {
    fn ingest_grib_idx(
        &mut self,
        idx_path: object_store::path::Path,
        idx_contents: &[u8],
    ) -> Result;
    fn manifest_as_ref(&self) -> &Manifest;
}
