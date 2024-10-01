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

The integer indexes get passed to the `hypergrib` backend for xarray. (In the future, `hypergrib` may implement a [custom xarray index](https://docs.xarray.dev/en/stable/internals/how-to-create-custom-index.html), so we can avoid the redundant conversion to integer indexes and back to coordinate labels).

`hypergrib` will then:

1. Convert integer indicies back to coordinate labels by looking up the appropriate labels in `hypergrib`'s coords arrays.
2. Find the unique tuples of init date, init hour, ensemble member, and step. Algorithmically generate the location of all the `.idx` files we need. For example, the GEFS location strings look like this: `
noaa-gefs-pds/gefs.<init date>/<init hour>/pgrb2b/gep<ensemble member>.t<init hour>z.pgrb2af<step>`
3. In parallel, submit GET requests for all these `.idx` files.
4. As soon as an `.idx` file arrives, decode it, and look up byte ranges of the GRIB files we need, and immediately submit GET requests for those byte ranges of the GRIB file. (This step is probably so fast that we perhaps don't need to multi-thread this... for the MVP, let's use a single thread for decoding `.idx` files and if that's too slow then we can add more threads). Maybe stop decoding rows in the `.idx` file once we've found all the metadata we need.
5. If an `.idx` file doesn't exist then:
    - Allow the user to determine what happens if `hypergrib` _tries_ but fails to read an `.idx` file. Three options: 
    - Silent: Don't complain about the missing `.idx`. Just load the GRIB, scan it, and keep in mem (because we'll soon extract binary data from it).
    - Warn: Log a warning about the missing `.idx`. And load the GRIB, scan it, and keep in mem.
    - Fail: Complain loudly about the missing `.idx`! Don't load the GRIB.
    - (Maybe, in a future version, we could offer the option to generate and cache `.idx` files locally)
7. If no GRIB exists then log another warning and insert the MISSING DATA indicator into the array (which will probably be NaN for floating point data).
6. As soon as GRIB data arrives, decode it, and place it into the final array. Decoding GRIB data should be multi-threaded.

## Enhancement: Allow the user to specify whether to load `.idx` files
Allow the user to set a threshold for when to load `.idx` files.

If the user requests more than THRESHOLD% of the GRIB messages in any GRIB file then skip the `.idx` and just load the GRIB. Otherwise, attempt to load the `.idx`. (The motivation being that, if the user wants to read most of the GRIB file, then loading the `.idx` first will add unnecessary latency).

Set the threshold to 100% to always try to load the `.idx` file before the GRIB.

Set the threshold to 0% to never load the `.idx`, and always load the GRIB file first.


## If it's too slow to get `.idx` files:

- For small GRIB files, just read the entirety of each GRIB file?
- Store `.idx` files locally?
- Convert `.idx` files to a more concise and cloud-friendly file format, which is published in a cloud bucket?
- Put all the `.idx` data into a cloud-side database?
- Put all the `.idx` data into a local database? DuckDB?
- We probably want to avoid using a manifest file, or putting metadata for every GRIB message into a database, because we want to scale to datasets with _trillions_ of GRIB messages. See https://github.com/JackKelly/hypergrib/discussions/14