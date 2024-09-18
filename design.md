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

We can compute the GRIB filename from the init_time, ensemble_member, and forecast step:

```
noaa-gefs-pds/gefs.YYYYMMDD/<init hour>/pgrb2b/gep<ensemble member>.t<init hour>z.pgrb2af<step>
```

`hypergrib` caches the information in the `.idx` files in a `BTreeMap`.

To create a dataset or to add new information to a dataset:

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Key {
  init_time: Datetime,
  ensemble_member: u16,  // Or an `enum EnsembleMember {Control, Perturbed(u16)}`
  forecast_step: Timedelta,
  nwp_variable: Varable,
  vertical_level: VerticalLevel,
}

// If the `derive` doesn't work then we can manually implement Ord, something like this:
impl Ord for Key {
  fn cmp(&self, other: &Self) -> Ordering {
    match self.init_time.cmp(&other.init_time) {
      Equal => {
        match self.ensemble_member.cmp(&other.ensemble_member) {
          Equal => {
            ...
          },
          m => m,
        }
      },
      m => m,
    }
  }
}

struct OffsetAndLen {
  byte_offset: u32,
  msg_length: u32,
}

struct CoordLabels {
  // We're using `Vector` (not `BTreeSet`) because the most performance-sensitive
  // part of the process is looking up a coord label given an integer index.
  // And the only way to do that with a `BTreeSet` is to first iterate over the elements.
  init_time: Vec<Datetime>,
  ensemble_member: Vec<u16>,
  forecast_step: Vec<Timedelta>,
  nwp_variable: Vec<Variable>,
  vertical_level: Vec<VerticalLevel>,
}

struct Dataset {
  coord_labels: CoordLabels,
  manifest: BTreeMap<Key, OffsetAndLen>,
  // Maybe we also want a `manifest_index` which maps integer indexes to `OffsetAndLen`
  // but let's make a start with the design below and benchmark it.
}

impl Dataset {
  fn insert(&mut self, key: Key, offset_and_len: OffsetAndLen) -> Result<(), AlreadyExistsError> {
    // Insert into `manifest` and update `coord_labels` iff the new coord doesn't exist yet.
  }

  fn coord_labels_to_offset_and_len(&self, key: &Key) -> Option<OffsetAndLen> {
    self.manifest[key]
  }

  fn index_locs_to_key(&self, index: &[u64]) -> Option<Key> {
    // get key by looking up the appropriate coord labels in self.coord_labels.
    // Returns `None` if any index is out of bounds (which is the same semantics as `Vec::get`).
    // Although maybe it'd be better to return a custom `Error` so we can say which dim
    // is out of bounds?
    Some(key)
  }
}

// GEFS-specific code:
fn key_to_gefs_filename(key: &Key) -> Path {
  // TODO
}

fn gefs_filename_to_key(path: &Path) -> Key {
  // TODO
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
