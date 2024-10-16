## Step 1: Create metadata describing each NWP datasets

The intention is that end-users wouldn't have to do this step. Instead an organisation (e.g. Open Climate Fix and/or dynamical.org) would perform this step and publish the metadata.

### Planned `hypergrib` MVP features (for getting coordinate labels)
- [ ] Get a list of init datetimes, ensemble members, and steps:
    1. List all `.idx` filenames. (Actually, we can get all the init datetimes "just" by reading the directory names perhaps using [`ObjectStore::list_with_delimiter`]([url](https://docs.rs/object_store/latest/object_store/trait.ObjectStore.html#tymethod.list_with_delimiter)) which is _not_ recursive. Although note that the docs for [`object_store::ListResult`]([url](https://docs.rs/object_store/latest/object_store/struct.ListResult.html)) say "_Individual result sets may be limited to 1,000 objects based on the underlying object storage’s limitations._". If this is too slow or doesn't work then we may have to use the native cloud storage APIs: for example, GCP [includes a `matchGlob` query parameter]([url](https://cloud.google.com/storage/docs/json_api/v1/objects/list)) that could be very useful. Or maybe work to extend `object_store` to implement `match_glob`)
    2. GEFS filenames include init datetimes, ensmeble members, and steps. Although this will be a dataset-specific feature. Other datasets might not include all this info in the filenames.
    3. Extract the init datetime (as a `DateTime<Utc>`), the ensemble member (as a `String`) and the step (as a `TimeDelta`).
- [ ] Record if/when the number of ensemble members and/or steps changes.
- [ ] Get a list of parameters and vertical levels by reading the bodies of first day's worth of `.idx` files, and the bodies of the last day's worth of `.idx` files. Beware that, for example, the GEFS analysis step doesn't include the same parameters as the forecast steps! (Which is why it's important to read an entire day's worth of data). If the first and last days have the same coordinate labels then assume that the coordinate labels stay the same across the entire dataset. If the coords in the first and last days differ then begin an "over-eager" binary search of the `.idx` files to find when coordinates change (e.g. when the NWP is upgraded an more ensemble members are added - see https://github.com/JackKelly/hypergrib/discussions/15). Submit many GET requests at once. The coords might change more than once. For the MVP:
    - Don't decode the parameter abbreviation string or the vertical level string. Keep these as strings. The coordinate labels will be these strings.
    - Ignore step in the `.idx` file. It's easier to get the `step` from the filename! (for GEFS, at least!)
- [ ] Get the horizontal spatial coordinates: Read a day's worth of GRIB files at the start of the dataset, and a day's worth at the end of the dataset. If the start and end of the dataset have the same coordinates then we're done; let's assume the spatial coords stay the same across the dataset. Otherwise conduct a kind of over-eager binary search to find exactly where the horizontal spatial coords change. The coords might change more than once. If the scan of `.idx` files found that the ensemble members and/or vertical levels changed then there's a good chance that the spatial coords also changed at the same times.
- [ ] Record the dimension names, array shape, and coordinates in a JSON file. Also record when the coordinates change. Changes in horizontal resolution probably have to be loaded as different xarray datasets (see https://github.com/JackKelly/hypergrib/discussions/15 and https://github.com/JackKelly/hypergrib/discussions/17).

### Features beyond the MVP
- [ ] Implement an efficient way to _update_ the `hypergrib` metadata (e.g. when NODD publishes new forecasts).
- [ ] Decode the parameter abbreviation string and the level string (so the user gets more information about what these mean, and so the levels can be put into order), possibly using the [GRIB tables represented as `.csv` files in GDAL](https://github.com/OSGeo/gdal/tree/master/frmts/grib/data) (See [the README for that directory](https://github.com/OSGeo/gdal/blob/master/frmts/grib/degrib/README.TXT), and [this gribberish discussion](https://github.com/mpiannucci/gribberish/issues/41#issuecomment-2404916278), and [my post to the gdal-dev mailing list](https://lists.osgeo.org/pipermail/gdal-dev/2024-October/059612.html) about splitting the CSVs and/or me copying the CSVs.) The current plan is to first generate Rust code in `gribberish` representing the GRIB tables (see https://github.com/mpiannucci/gribberish/issues/63), and then parse `.idx` files into these `gribberish` data structures.
    - [ ] Also need to decode `.idx` parameter strings like this (from HRRR): `var discipline=0 center=7 local_table=1 parmcat=16 parm=201`
- [ ] Open other GRIB datasets. (If we have to parse the step from the body of `.idx` files then consider using [`nom`](https://crates.io/crates/nom)).
- [ ] Optimise the extraction of the horizontal spatial coords from the GRIBs by only loading the relevant sections from the GRIBs (using the `.idx` files). Although this optimisation isn't urgent. Users will never have to run this step.

## Step 2: Load the metadata and load data

Open dataset:

```python
da = xr.open_dataset(URL, engine="hypergrib")
```

`hypergrib` loads the metadata and passes to xarray the full list of coordinates and dimension names, e.g.:

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

### Planned `hypergrib` MVP features:

- [ ] Load the `hypergrib` metadata (which was produced by step 1).
- [ ] Convert integer indicies back to coordinate labels by looking up the appropriate labels in `hypergrib`'s coords arrays.
- [ ] Find the unique tuples of init date, init hour, ensemble member, and step.
- [ ] Algorithmically generate the location of all the `.idx` files we need. For example, the GEFS location strings look like this: `
noaa-gefs-pds/gefs.<init date>/<init hour>/pgrb2b/gep<ensemble member>.t<init hour>z.pgrb2af<step>`
- [ ] In parallel, submit GET requests for all these `.idx` files.
- [ ] As soon as an `.idx` file arrives, decode it, and look up byte ranges of the GRIB files we need, and immediately submit GET requests for those byte ranges of the GRIB file. (This step is probably so fast that we perhaps don't need to multi-thread this... for the MVP, let's use a single thread for decoding `.idx` files and if that's too slow then we can add more threads). Maybe stop decoding rows in the `.idx` file once we've found all the metadata we need.
- [ ] If an `.idx` file doesn't exist then:
    - Allow the user to determine what happens if `hypergrib` _tries_ but fails to read an `.idx` file. Three options: 
    - Silent: Don't complain about the missing `.idx`. Just load the GRIB, scan it, and keep in mem (because we'll soon extract binary data from it).
    - Warn: Log a warning about the missing `.idx`. And load the GRIB, scan it, and keep in mem.
    - Fail: Complain loudly about the missing `.idx`! Don't load the GRIB.
    - (Maybe, in a future version, we could offer the option to generate and cache `.idx` files locally)
- [ ] If no GRIB exists then log another warning and insert the MISSING DATA indicator into the array (which will probably be NaN for floating point data).
- [ ] As soon as GRIB data arrives, decode it, and place it into the final array. Decoding GRIB data should be multi-threaded.
- [ ] Benchmark! See recent discussion on "[Large Scale Geospatial Benchmarks](https://discourse.pangeo.io/t/large-scale-geospatial-benchmarks/4498/2)" on the Pangeo forum.

### Features beyond the MVP

#### Allow the user to specify whether to load `.idx` files
Allow the user to set a threshold for when to load `.idx` files.

If the user requests more than THRESHOLD% of the GRIB messages in any GRIB file then skip the `.idx` and just load the GRIB. Otherwise, attempt to load the `.idx`. (The motivation being that, if the user wants to read most of the GRIB file, then loading the `.idx` first will add unnecessary latency).

Set the threshold to 100% to always try to load the `.idx` file before the GRIB.

Set the threshold to 0% to never load the `.idx`, and always load the GRIB file first.

#### Define an extended idx format
See https://github.com/JackKelly/hypergrib/discussions/17

#### Schedule the network IO to balance several different objectives:
- Keep a few hundred network request in-flight at any given moment (user configurable). (Why? Because the [AnyBlob paper](https://www.vldb.org/pvldb/vol16/p2769-durner.pdf) suggests that is what's required to achieve max throughput).
- Consolidate nearby byterange requests (user configurable) to minimise overhead, and reduce the total number of IO operations.

#### Slice _into_ each GRIB message
For example, some GRIBs are compressed in JPEG2000, and JPEG2000 allows _parts_ of the image to be decompressed. And maybe, whilst making the manifest, we could decompress each GRIB file and save the state of the decompressor every, say, 4 kB. Then, at query time, if we want a single pixel then we'd have to stream at most 4 kB of data from disk. Although that has its own issues.).

## If it's too slow to get `.idx` files:

- For small GRIB files, just read the entirety of each GRIB file?
- Store `.idx` files locally?
- Convert `.idx` files to a more concise and cloud-friendly file format, which is published in a cloud bucket?
- Put all the `.idx` data into a cloud-side database?
- Put all the `.idx` data into a local database? DuckDB?
- We probably want to avoid using a manifest file, or putting metadata for every GRIB message into a database, because we want to scale to datasets with _trillions_ of GRIB messages. See https://github.com/JackKelly/hypergrib/discussions/14
