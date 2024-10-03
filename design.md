## Step 1: Create metadata describing each NWP datasets

The intention is that end-users wouldn't have to do this step. Instead an organisation (e.g. Open Climate Fix and/or dynamical.org) would perform this step and publish the metadata.

### Planned `hypergrib` MVP features:
1. [ ] Implement parsing of `.idx` files into `gribberish` data structures (starting with the GEFS NWP) https://github.com/JackKelly/hypergrib/issues/11
2. [ ] Get coordinate labels for init datetimes, vertical levels, parameters, steps, and ensemble members:
    - Get a list of init datetimes, ensemble members, and steps by getting a listing of all `.idx` filenames. Record if/when the coordinates change. The GEFS filenames include all this information. Although this will be a dataset-specific feature. Other datasets might not include all this info in the filenames.
    - Get a list of parameters and vertical levels by reading the first day's worth of `.idx` files, and read the last day's worth of `.idx` files. Beware that, for example, the GEFS analysis step doesn't include the same parameters as the forecast steps! If the first and last days have the same coordinate labels then assume that the coordinate labels stay the same across the entire dataset. If the coords in the first and last days differ then begin an "over-eager" binary search of the `.idx` files to find when coordinates change (e.g. when the NWP is upgraded an more ensemble members are added - see https://github.com/JackKelly/hypergrib/discussions/15). Submit many GET requests at once. The coords might change more than once.
3. [ ] Get the horizontal spatial coordinates: Read a day's worth of GRIB files at the start of the dataset, and a day's worth at the end of the dataset. If the start and end of the dataset have the same coordinates then we're done; let's assume the spatial coords stay the same across the dataset. Otherwise conduct a kind of over-eager binary search to find exactly where the horizontal spatial coords change. The coords might change more than once.
4. [ ] Record the dimension names, array shape, and coordinates in a JSON file. Also record when the coordinates change. Changes in horizontal resolution probably have to be loaded as different xarray datasets (see https://github.com/JackKelly/hypergrib/discussions/15).

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

1. [ ] Convert integer indicies back to coordinate labels by looking up the appropriate labels in `hypergrib`'s coords arrays.
2. [ ] Find the unique tuples of init date, init hour, ensemble member, and step.
3. [ ] Algorithmically generate the location of all the `.idx` files we need. For example, the GEFS location strings look like this: `
noaa-gefs-pds/gefs.<init date>/<init hour>/pgrb2b/gep<ensemble member>.t<init hour>z.pgrb2af<step>`
4. [ ] In parallel, submit GET requests for all these `.idx` files.
5. [ ] As soon as an `.idx` file arrives, decode it, and look up byte ranges of the GRIB files we need, and immediately submit GET requests for those byte ranges of the GRIB file. (This step is probably so fast that we perhaps don't need to multi-thread this... for the MVP, let's use a single thread for decoding `.idx` files and if that's too slow then we can add more threads). Maybe stop decoding rows in the `.idx` file once we've found all the metadata we need.
6. [ ] If an `.idx` file doesn't exist then:
    - Allow the user to determine what happens if `hypergrib` _tries_ but fails to read an `.idx` file. Three options: 
    - Silent: Don't complain about the missing `.idx`. Just load the GRIB, scan it, and keep in mem (because we'll soon extract binary data from it).
    - Warn: Log a warning about the missing `.idx`. And load the GRIB, scan it, and keep in mem.
    - Fail: Complain loudly about the missing `.idx`! Don't load the GRIB.
    - (Maybe, in a future version, we could offer the option to generate and cache `.idx` files locally)
7. [ ] If no GRIB exists then log another warning and insert the MISSING DATA indicator into the array (which will probably be NaN for floating point data).
8. [ ] As soon as GRIB data arrives, decode it, and place it into the final array. Decoding GRIB data should be multi-threaded.

### Features beyond the MVP

#### Allow the user to specify whether to load `.idx` files
Allow the user to set a threshold for when to load `.idx` files.

If the user requests more than THRESHOLD% of the GRIB messages in any GRIB file then skip the `.idx` and just load the GRIB. Otherwise, attempt to load the `.idx`. (The motivation being that, if the user wants to read most of the GRIB file, then loading the `.idx` first will add unnecessary latency).

Set the threshold to 100% to always try to load the `.idx` file before the GRIB.

Set the threshold to 0% to never load the `.idx`, and always load the GRIB file first.

#### Define an extended idx format
- "Sequences of GRIB sections 2 to 7, sections 3 to 7 or sections 4 to 7 may be repeated within a single GRIB message" (quoted from the [WMO Manual on Codes](https://library.wmo.int/viewer/35625/download?file=WMO-306-v-I-2-2023_en.pdf)). Describe the byte offset of each data section? Because we want to allow users to load only the data sections that they need.
- Include the message length in each row!
- Describe the coord labels, dim names, and when the coord labels change in the metadata.

#### Schedule the network IO to balance several different objectives:
- Keep a few hundred network request in-flight at any given moment (user configurable). (Why? Because the [AnyBlob paper](https://www.vldb.org/pvldb/vol16/p2769-durner.pdf) suggests that is what's required to achieve max throughput).
- Consolidate nearby byterange requests (user configurable) to minimise overhead, and reduce the total number of IO operations.


## If it's too slow to get `.idx` files:

- For small GRIB files, just read the entirety of each GRIB file?
- Store `.idx` files locally?
- Convert `.idx` files to a more concise and cloud-friendly file format, which is published in a cloud bucket?
- Put all the `.idx` data into a cloud-side database?
- Put all the `.idx` data into a local database? DuckDB?
- We probably want to avoid using a manifest file, or putting metadata for every GRIB message into a database, because we want to scale to datasets with _trillions_ of GRIB messages. See https://github.com/JackKelly/hypergrib/discussions/14
