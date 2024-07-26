# hypergrib
Lazily read petabytes of GRIBs from cloud object storage.

The ultimate aim is very much inspired by [kerchunk](https://fsspec.github.io/kerchunk/), [VirtualiZarr](https://github.com/zarr-developers/VirtualiZarr), and [dynamical.org](https://dynamical.org): Opening a multi-petabyte GRIB dataset from cloud object storage should be as simple as:

```python
dataset = xarray.open_dataset(URL)
```

`hypergrib` is focused on performance: If you're using a VM with a 200 Gbps NIC, in the same region as the data, then you should be able to read GRIBs at ~20 GBytes per second. And each load should incur minimal latency.

The ultimate dream is to be able to train large machine learning models directly from GRIBs on cloud object storage, such as the petabytes of GRIB files shared by the [NOAA Open Data Dissemination](https://www.noaa.gov/nodd) (NODD) programme, [ECMWF](https://www.ecmwf.int/en/forecasts/datasets/open-data), and others.

> **Note**
> This code is at its very earliest stage! It won't do anything useful for a while!

Why does `hypergrib` exist? At least to start with, `hypergrib` is very much an experiment (which stands on the shoulders of giants like gribberish, kerchunk, Zarr, xarray, etc.). The question we're asking with this experiment is: How fast can we go if we "cheat" and build a very special-purpose tool which is focused just on reading multi-file GRIBs from cloud object storage. Let's throw in all the performance tricks we can think of. Perhaps, for a bunch of use-cases, reading directly from GRIBs will be sufficient. Although there will definitely be read-patterns which will never be well-served by reading from GRIBs, and the data will have to be converted to something like Zarr.

## Planned features
- [ ] Create a very concise [manifest](https://github.com/JackKelly/hypergrib/issues/1) from GRIB `.idx` files (ultimately, this manifest file would be shared publicly, so most users only have to run `xr.open_dataset(MANIFEST_URL)` to load a petabyte-scale GRIB dataset).
- [ ] Lazily open the multi-GRIB dataset by reading the manifest
- [ ] Load just the GRIB data that's required. Read and process as little data as possible.
- [ ] Keep a few hundred network request in-flight at any given moment (user configurable). (Why? Because the [AnyBlob paper](https://www.vldb.org/pvldb/vol16/p2769-durner.pdf) suggests that is what's required to achieve max throughput).
- [ ] Consolidate nearby byterange requests (user configurable).
- [ ] Simple Python API (probably using asyncio)
- [ ] Integrate with xarray
- [ ] Efficiently update the manifest when new GRIBs arrive
- [ ] Convert the `hypergrib` manifest to and from kerchunk / VirtualiZarr / Zarr manifest files
