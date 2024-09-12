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

xarray converts these coordinated labels to integer indexes:

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

In `hypergrib` we standardise the interface to different NWPs:

```rust
trait NWP {
  fn get_filename(&self) -> Path;
  fn get_byte_offset_and_len(&self) -> ByteOffsetAndLen;
}
```

We can compute the GRIB filename from the init_time, ensemble_member, and forecast step:

```
noaa-gefs-pds/gefs.YYYYMMDD/<init hour>/pgrb2b/gep<ensemble member>.t<init hour>z.pgrb2af<step>
```

`hypergrib` will cache the information in the `.idx` files in a `BTreeMap`:

```rust
struct Key {
  init_time,
  ensemble_member,
  forecast_step,
  nwp_variable,
  vertical_level,
}

impl Key {
  fn to_filename(&self) -> Path {
    // implementation
  }
}

struct ByteOffsetAndLength {
  byte_offset: u32,
  msg_length: u32,
}

struct GEFS {
  manifest: BTreeMap<Key, ByteOffsetAndLength>,
}

impl NWP for GEFS {
  impl get_filename() // ...need to think more about how this'll work!
}

```

To satisfy the user's query, we'll loop round all the requested positions, and build a `BTreeMap<filename, Vec<ByteOffsetAndLen>>`. Which we then grab from storage.
