#![doc = include_str!("../README.md")]

pub mod datasets;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, TimeDelta, Utc};

// TODO: Replace this with Enums from gribberish.
#[derive(PartialEq, Eq, Hash, Clone)]
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
#[derive(PartialEq, Eq, Hash, Clone)] // PartialEq, Eq, and Hash are required for HashMap keys.
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
#[derive(PartialEq, Eq, Hash, Clone)] // PartialEq, Eq, and Hash are required for HashMap keys.
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

#[derive(PartialEq, Eq, Hash, Clone)] // PartialEq, Eq, and Hash are required for HashMap keys.
struct Key {
    reference_time: DateTime<Utc>,
    ensemble_member: EnsembleMember,
    forecast_step: TimeDelta,
    parameter: Parameter,
    vertical_level: VerticalLevel,
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
    fn new(base_path: object_store::path::Path) -> Self {
        Self {
            base_path,
            paths: HashSet::new(),
            manifest: HashMap::new(),
        }
    }

    /// Adds a (key, msg_loc) pair to the manifest.
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the manifest did not previously contain this value, `true` is returned.
    /// - If the manifest already contained this value, `false` is returned,
    ///   and the set is not modified: original value is not replaced,
    ///   and the value passed as argument is dropped.
    fn insert(
        &mut self,
        key: Key,
        path: object_store::path::Path,
        byte_offset: u32,
        msg_length: u32,
    ) -> bool {
        // TODO: Update `self.coord_labels` if necessary.
        if self.manifest.contains_key(&key) {
            return false;
        };
        let path_arc = if let Some(pa) = self.paths.get(&path) {
            pa.clone()
        } else {
            let pa = Arc::new(path);
            self.paths.insert(pa.clone());
            pa
        };
        let msg_loc = MessageLocation {
            path: path_arc,
            byte_offset,
            msg_length,
        };
        assert!(self.manifest.insert(key, msg_loc).is_none());
        true
    }

    fn as_ref(&self) -> &HashMap<Key, MessageLocation> {
        &self.manifest
    }

    fn index_locations_to_key(&self, index: &[u64]) -> Option<&Key> {
        // get key by looking up the appropriate coord labels in self.coord_labels.
        // Returns `None` if any index is out of bounds (which is the same semantics as `Vec::get`).
        // Although maybe it'd be better to return a custom `Error` so we can say which dim
        // is out of bounds? Or if there are the wrong number of dims in the `index`?
        todo!()
    }
}

trait Dataset {
    fn ingest_grib_idx(
        &mut self,
        idx_path: object_store::path::Path,
        idx_contents: &[u8],
    ) -> anyhow::Result<()>;
    fn manifest_as_ref(&self) -> &Manifest;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_manifest() -> Manifest {
        let base_path = object_store::path::Path::from("/foo/bar");
        Manifest::new(base_path)
    }

    #[test]
    fn test_new_manifest() {
        new_manifest();
    }

    #[test]
    fn test_insert() -> anyhow::Result<()> {
        let mut manifest = new_manifest();
        let path1 = object_store::path::Path::from("/baz/01");
        let key1 = Key {
            reference_time: DateTime::parse_from_rfc3339("1996-12-19T16:00:00+00:00")
                .unwrap()
                .to_utc(),
            ensemble_member: EnsembleMember::Perturbed(1),
            forecast_step: TimeDelta::zero(),
            parameter: Parameter::Temperature_K,
            vertical_level: VerticalLevel::MeanSeaLevel,
        };
        assert!(manifest.insert(key1.clone(), path1.clone(), 0, 4000));
        assert_eq!(manifest.as_ref().len(), 1);
        assert_eq!(
            Arc::strong_count(&manifest.as_ref().get(&key1).unwrap().path),
            2
        );

        // Check that attempting to insert the same key again returns false:
        assert!(!manifest.insert(key1.clone(), path1.clone(), 0, 4000));
        assert_eq!(manifest.as_ref().len(), 1);
        assert_eq!(
            Arc::strong_count(&manifest.as_ref().get(&key1).unwrap().path),
            2
        );

        // Insert a second key, with the same path
        let key2 = Key {
            ensemble_member: EnsembleMember::Control,
            ..key1
        };
        assert!(manifest.insert(key2.clone(), path1.clone(), 50, 5000));
        assert_eq!(manifest.as_ref().len(), 2);
        assert_eq!(
            Arc::strong_count(&manifest.as_ref().get(&key2).unwrap().path),
            3
        );
        Ok(())
    }
}
