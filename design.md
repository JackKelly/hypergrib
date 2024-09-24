Open dataset:

```python
da = xr.open_dataset(URL, engine="hypergrib")
```

`hypergrib` loads the manifest and passes to xarray the full list of coordinates and dimension names, e.g.:

```python
dims = ["init_time", "variable", "vertical_level", "timestep", "ensemble_member"]
coords = {
  "init_time": ["2024-01-01", "2024-01-02"],
  # etc.
}
```

User request: 

```python
da.sel(
  init_time="2024-01-01",
  nwp_variable=["temperature", "wind_speed"],
  vertical_level="2meters",
  # all forecast time steps
  # all ensemble members
)
```

xarray converts these coordinate labels to integer indexes:

```python
da.isel(
  init_time=0,
  nwp_variable=[0, 1],
  vertical_level=0,
  # all forecast time steps
  # all ensemble members
)
```

And `hypergrib` needs to load the appropriate data for these integer indexes.

In `hypergrib` we standardise the interface to different datasets:

```
noaa-gefs-pds/gefs.YYYYMMDD/<init hour>/pgrb2b/gep<ensemble member>.t<init hour>z.pgrb2af<step>
```

`hypergrib` caches the information in the `.idx` files in a `HashMap`.

To create a dataset or to add new information to a dataset:

```rust
#[derive(PartialEq, Eq, Hash)] // PartialEq, Eq, and Hash are required for HashMap keys.
struct NwpKey {
  init_time: Datetime,
  ensemble_member: u16,  // Or an `enum EnsembleMember {Control, Perturbed(u16)}`
  forecast_step: Timedelta,
  nwp_variable: Variable,  // `Variable` is an enum
  vertical_level: VerticalLevel, // `VerticalLevel` is an enum?
  // Also for consideration:
  // provider: Provider,  // e.g. NOAA, UKMetOffice, ECMWF, etc.
  // nwp_model: NWPModel,  // e.g. GFS, GEFS, UKV, etc.
  // or maybe combine `provider` and `nwp_model` into a single Enum e.g. UKMO_UKV, etc?
}

struct Chunk<'p> {
  path: &'p object_store::Path,
  byte_offset: u32,
  msg_length: u32,
}

struct SortedVecSet<T>(Vec<T>);

impl<T> SortedVecSet<T> {
  /// Insert only if a duplicate doesn't exist. Sorts after insertion.
  fn insert(t: T) -> Result<DuplicateExists>;
}

struct NwpCoordLabels {
  // We're using `SortedVecSet` (not `BTreeSet`) because the most performance-sensitive
  // part of the process is looking up a coord label given an integer index.
  // And the only way to do that with a `BTreeSet` is to first iterate over the elements.
  init_time: SortedVecSet<Datetime>,
  ensemble_member: SortedVecSet<u16>,
  forecast_step: SortedVecSet<Timedelta>,
  nwp_variable: SortedVecSet<Variable>,
  vertical_level: SortedVecSet<VerticalLevel>,
}

// Or maybe this doesn't have to be generic over K and COORDS? Will all datasets use the same key and coords?
struct Dataset<K, COORDS>
where
  K: PartialEq + Eq + Hash,
  {
  coord_labels: COORDS,
  // Need to store the paths once, so we only store a reference to each Path
  // in the `Chunk`. Each path in `paths` will be relative to `base_path`.
  paths: HashSet<object_store::Path>,
  manifest: HashMap<K, Chunk>,
  // Maybe we also want a `manifest_index` which maps integer indexes to `Chunk`
  // but let's make a start with the design below and benchmark it.
}

impl<K, COORDS> Dataset<K, COORDS> {
  fn insert(&mut self, key: K, chunk: Chunk) -> Result<(), AlreadyExistsError> {
    // Insert into `manifest` and update `coord_labels` iff the new coord doesn't exist yet.
  }

  fn coord_labels_to_chunk(&self, key: &K) -> Option<Chunk> {
    self.manifest[key]
  }

  fn index_locs_to_key(&self, index: &[u64]) -> Option<K> {
    // get key by looking up the appropriate coord labels in self.coord_labels.
    // Returns `None` if any index is out of bounds (which is the same semantics as `Vec::get`).
    // Although maybe it'd be better to return a custom `Error` so we can say which dim
    // is out of bounds? Or if there are the wrong number of dims in the `index`?
    Some(key)
  }
}

trait NwpDataset {
  fn ingest_grib_idx(&mut self, idx_path: Path, idx_contents: &[u8]) -> Result;
}

// GEFS-specific code
struct GefsDataset {
  dataset: Dataset<NwpKey, NwpCoordLabels>,
}

impl NwpDataset for GefsDataset {
  fn ingest_grib_idx(&mut self, idx_path: Path, idx_contents: &[u8]) -> Result {
    // insert `idx_path` into `self.dataset.paths`, and get a ref to the `path` in `paths`
    // for use in the `Chunk`.
  }

  fn as_ref(&self) -> &Dataset<NwpKey, NwpCoordLabels> {
    &self.dataset
  }
}

```


To query a dataset:

```rust

// Usage example: Create manifest
// TODO

// Usage example: Convert from coordinate labels to integer indexes

// Usage example: Get GRIB message for a given key (using labels)
 
```

To satisfy the user's query, we'll loop round all the requested positions, and build a `BTreeMap<filename, Vec<ByteOffsetAndLen>>`. Which we then grab from storage.
