# hypergrib

> **Warning**
> This code is at its very earliest stage! It won't do anything useful for a while!

Lazily read petabytes of GRIBs from cloud object storage, as fast as the hardware will allow.

The ultimate aim is very much inspired by [kerchunk](https://fsspec.github.io/kerchunk/), [VirtualiZarr](https://github.com/zarr-developers/VirtualiZarr), and [dynamical.org](https://dynamical.org): Opening a multi-petabyte GRIB dataset from cloud object storage should be as simple as:

```python
dataset = xarray.open_dataset(URL)
```

`hypergrib` is focused on performance, especially for random-access: A VM with a 200 Gbps NIC in the same region as the data should be able to read GRIBs at ~20 gigabytes per second from object storage. And each load should incur minimal latency.

The ultimate dream is to be able to train large machine learning models directly from GRIBs on cloud object storage, such as the petabytes of GRIB files shared by the [NOAA Open Data Dissemination](https://www.noaa.gov/nodd) (NODD) programme, [ECMWF](https://www.ecmwf.int/en/forecasts/datasets/open-data), and others.

Why does `hypergrib` exist? At least to start with, `hypergrib` is very much an experiment (which stands on the shoulders of giants like gribberish, kerchunk, Zarr, xarray, etc.). The question we're asking with this experiment is: How fast can we go if we "cheat" by building a _special-purpose_ tool focused on reading multi-file GRIBs from cloud object storage. Let's throw in all the performance tricks we can think of. And let's also bake in a bunch of domain knowledge about GRIBs. We're explicitly _not_ trying to build a general-purpose tool like kerchunk, so we can build something that's as lean & focused as possible.

Reading directly from GRIBs will probably be sufficient for a bunch of use-cases.

On the other hand, there will definitely be read-patterns which will never be well-served by reading from GRIBs, and the data will have to be converted to something like Zarr. For example, reading a long timeseries for a single geographical point will involve reading about one million times more data off disk than you need (assuming each 2D GRIB message is 1,000 x 1,000 pixels).

For more info, please see [this draft blog post](https://docs.google.com/document/d/1IHoAY3hnAu4aCJ1Vb62lQHI_GmIcMYMTkdM-nUbjmQ0).

## Planned features
- [ ] Create a very concise [manifest](https://github.com/JackKelly/hypergrib/issues/1) from GRIB `.idx` files (ultimately, this manifest file would be shared publicly, so most users would only have to run `xr.open_dataset(MANIFEST_URL)` to lazily open a petabyte-scale GRIB dataset).
  - [ ] We'll probably start with the GEFS NWP
- [ ] Lazily open the multi-GRIB dataset by reading the manifest
- [ ] Load just the GRIB data that's required. Read and process as little data as possible. Maybe even go as far as _just_ decompressing part of each GRIB message.
- [ ] Process GRIBs in parallel across multiple CPU cores.
- [ ] Schedule the network IO to ballance several different objectives:
  - [ ] Keep a few hundred network request in-flight at any given moment (user configurable). (Why? Because the [AnyBlob paper](https://www.vldb.org/pvldb/vol16/p2769-durner.pdf) suggests that is what's required to achieve max throughput).
  - [ ] Consolidate nearby byterange requests (user configurable) to minimise overhead, and reduce the total number of IO operations.
- [ ] Simple Python API (probably using asyncio)
- [ ] Integrate with xarray
- [ ] Run a service to efficiently update the manifests when new GRIBs arrive
- [ ] Convert the `hypergrib` manifest to and from kerchunk / VirtualiZarr / Zarr manifest files

## Name

`hypergrib` uses "hyper" in its mathematical sense, like [hypercube](https://en.wikipedia.org/wiki/Hypercube) (an n-dimensional cube). Oh, and it's reminiscent of a very cool record label, too :)
