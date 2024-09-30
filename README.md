# hypergrib

> [!WARNING]
> This code is at its very earliest stage! It won't do anything useful for a while!

Lazily read petabytes of [GRIB](https://en.wikipedia.org/wiki/GRIB) files from cloud object storage, as fast as the hardware will allow.

This project is inspired by [kerchunk](https://fsspec.github.io/kerchunk/), [VirtualiZarr](https://github.com/zarr-developers/VirtualiZarr), and [dynamical.org](https://dynamical.org).

The aim is that opening a multi-petabyte GRIB dataset from cloud object storage should be as simple as:

```python
dataset = xarray.open_dataset(URL, engine="hypergrib")
```

`hypergrib` is focused on performance: A virtual machine with a 200 Gbps (gigabit per second) network interface card in the same region as the data should be able to read GRIBs at ~20 gigabytes per second from object storage. Each load should incur minimal latency. Random access should be as fast & efficient as possible.

The ultimate dream is to be able to train large machine learning models directly from GRIBs on cloud object storage, such as the petabytes of GRIB files shared by the [NOAA Open Data Dissemination](https://www.noaa.gov/nodd) (NODD) programme, [ECMWF](https://www.ecmwf.int/en/forecasts/datasets/open-data), and others.

Why does `hypergrib` exist? At least to start with, `hypergrib` is an experiment (which stands on the shoulders of giants like `gribberish`, `kerchunk`, `Zarr`, `xarray`, `VirtualiZarr` etc.). The question we're asking with this experiment is: How fast can we go if we "cheat" by building a _special-purpose_ tool focused on reading multi-file GRIB datasets from cloud object storage. Let's throw in all the performance tricks we can think of. And let's also bake in a bunch of domain knowledge about GRIBs. We're explicitly _not_ trying to build a general-purpose tool like the awesome `kerchunk`. If `hypergrib` is faster than existing approaches, then maybe ideas from `hypergrib` could be merged into existing tools, and `hypergrib` will remain a testing ground rather than a production tool. Or maybe `hypergrib` will mature into a tool that can be used in production.

Reading directly from GRIBs will probably be sufficient for a lot of use-cases.

On the other hand, there will definitely be read-patterns which will never be well-served by reading from GRIBs (because of the way the data is structured on disk). For example, reading a long timeseries for a single geographical point will involve reading about one million times more data from disk than you need (assuming each 2D GRIB message is 1,000 x 1,000 pixels). So, even if you sustain 20 gigabytes per second from GRIBs in object storage, you'll only get 20 _kilobytes_ per second of useful data! For these use-cases, the data will almost certainly have to be converted to something like Zarr. (And, hopefully, `hypergrib` will help make the conversion from GRIB to Zarr as efficient as possible).

(That said, we're keen to explore ways to slice _into_ each GRIB message... e.g. some GRIBs are compressed in JPEG2000, and JPEG2000 allows _parts_ of the image to be decompressed. And maybe, whilst making the manifest, we could decompress each GRIB file and save the state of the decompressor every, say, 4 kB. Then, at query time, if we want a single pixel then we'd have to stream at most 4 kB of data from disk. Although that has its own issues.).

For more info, please see [this blog post](https://openclimatefix.org/post/lazy-loading-making-it-easier-to-access-vast-datasets-of-weather-satellite-data).

## Planned features
- [ ] Parse `.idx` files (starting with the GEFS NWP).
- [ ] Read all `.idx` files to get a list of coordinate labels for each dimension. Submit many GET requests at once. Record if/when coordinates change (e.g. when the NWP is upgraded an more ensemble members are added - see https://github.com/JackKelly/hypergrib/discussions/15).
- [ ] Algorithmically generate filenames of `.idx` files from coord labels.
- [ ] Given a user query (expressed as integery index ranges for each dimension), convert to coord labels, and generate a list of `.idx` files to load.
- [ ] Submit GET requests in parallel for those `.idx` files.
- [ ] See [`design.md`](https://github.com/JackKelly/hypergrib/blob/main/design.md) for more tasks.
- [ ] Schedule the network IO to balance several different objectives:
  - [ ] Keep a few hundred network request in-flight at any given moment (user configurable). (Why? Because the [AnyBlob paper](https://www.vldb.org/pvldb/vol16/p2769-durner.pdf) suggests that is what's required to achieve max throughput).
  - [ ] Consolidate nearby byterange requests (user configurable) to minimise overhead, and reduce the total number of IO operations.
- [ ] Integrate with xarray
- [ ] Integrate with virtualizarr. See [VirtualiZarr/Add hypergrib as as a grib reader #238](https://github.com/zarr-developers/VirtualiZarr/issues/238)
- [ ] Benchmark! See recent discussion on "[Large Scale Geospatial Benchmarks](https://discourse.pangeo.io/t/large-scale-geospatial-benchmarks/4498/2)" on the Pangeo forum.

## Name
`hypergrib` uses "hyper" in its mathematical sense, like [hypercube](https://en.wikipedia.org/wiki/Hypercube) (an n-dimensional cube). Oh, and it's reminiscent of a very cool record label, too :)
