# hypergrib

> [!WARNING]
> This code is at a very early stage! It won't do anything useful for a while!

Lazily read petabytes of [GRIB](https://en.wikipedia.org/wiki/GRIB) files from cloud object storage, as fast as the hardware will allow.

This project is inspired by [kerchunk](https://fsspec.github.io/kerchunk/), [VirtualiZarr](https://github.com/zarr-developers/VirtualiZarr), and [dynamical.org](https://dynamical.org).

The aim is that opening a multi-petabyte GRIB dataset from cloud object storage should be as simple as:

```python
dataset = xarray.open_dataset(URL, engine="hypergrib")
```

`hypergrib` is focused on performance: A virtual machine with a 200 Gbps (gigabit per second) network interface card in the same region as the data should be able to read GRIBs at ~20 gigabytes per second from object storage. Each load should incur minimal latency. Random access should be as fast & efficient as possible.

The ultimate dream is to be able to train large machine learning models directly from GRIBs on cloud object storage, such as the petabytes of GRIB files shared by the [NOAA Open Data Dissemination](https://www.noaa.gov/nodd) (NODD) programme, [ECMWF](https://www.ecmwf.int/en/forecasts/datasets/open-data), and others.

## Goals
- Allow users to lazily open petabyte-scale NWP datasets from their laptop with a single line of code: `xr.open_dataset`.
- Scale to datasets with _trillions_ of GRIB messages (see https://github.com/JackKelly/hypergrib/discussions/14)
- Create and constantly update metadata for the main public NWP datasets (so users don't have to do this themselves).
- Performance: low latency and high bandwidth. A VM with a 200 Gbps NIC should sustain 20 gigabytes per sec from cloud object storage.
- Computational efficiency and "mechanical sympathy" with cloud object storage
- Integrate with:
    - xarray
    - virtualizarr. See [VirtualiZarr/Add hypergrib as as a grib reader #238](https://github.com/zarr-developers/VirtualiZarr/issues/238)

## Why does `hypergrib` exist?
At least to start with, `hypergrib` is an experiment (which stands on the shoulders of giants like `gribberish`, `kerchunk`, `Zarr`, `xarray`, `VirtualiZarr` etc.). The question we're asking with this experiment is: How fast can we go if we "cheat" by building a _special-purpose_ tool focused on reading multi-file GRIB datasets from cloud object storage. Let's throw in all the performance tricks we can think of. And let's also bake in a bunch of domain knowledge about GRIBs. We're explicitly _not_ trying to build a general-purpose tool like the awesome `kerchunk`. If `hypergrib` is faster than existing approaches, then maybe ideas from `hypergrib` could be merged into existing tools, and `hypergrib` will remain a testing ground rather than a production tool. Or maybe `hypergrib` will mature into a tool that can be used in production.

Reading directly from GRIBs will probably be sufficient for a lot of use-cases.

## Some read patterns will never be well-served by reading directly from GRIBs
There are read-patterns which will never be well-served by reading from GRIBs (because of the way the data is structured on disk). For example, reading a long timeseries for a single geographical point will involve reading about one million times more data from disk than you need (assuming each 2D GRIB message is 1,000 x 1,000 pixels). So, even if you sustain 20 gigabytes per second from GRIBs in object storage, you'll only get 20 _kilobytes_ per second of useful data! For these use-cases, the data will almost certainly have to be converted to something like Zarr. (And, hopefully, `hypergrib` will help make the conversion from GRIB to Zarr as efficient as possible).

(That said, we're keen to explore ways to slice _into_ each GRIB message... e.g. some GRIBs are compressed in JPEG2000, and JPEG2000 allows _parts_ of the image to be decompressed. And maybe, whilst making the manifest, we could decompress each GRIB file and save the state of the decompressor every, say, 4 kB. Then, at query time, if we want a single pixel then we'd have to stream at most 4 kB of data from disk. Although that has its own issues.).

## More info about `hypergrib`
For the planned design, please see [design.md](https://github.com/JackKelly/hypergrib/blob/main/design.md).

For more info on the background and motivation for `hypergrib`, please see [this blog post](https://openclimatefix.org/post/lazy-loading-making-it-easier-to-access-vast-datasets-of-weather-satellite-data).

## Name
`hypergrib` uses "hyper" in its mathematical sense, like [hypercube](https://en.wikipedia.org/wiki/Hypercube) (an n-dimensional cube). Oh, and it's reminiscent of a very cool record label, too :)

## GRIB2 documentation

1. [WMO Manual on Codes, Volume I.2, 2023 edition](https://library.wmo.int/viewer/35625/download?file=WMO-306-v-I-2-2023_en.pdf) - See overview diagram of GRIB messages on PDF page 21.
2. [wgrib C source code](https://github.com/NOAA-EMC/NCEPLIBS-grib_util/blob/develop/src/wgrib/wgrib.c)
3. [NCEP WMO GRIB2 Documentation](https://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_doc/)
4. [GRIB2 use at NCEP](https://www.nco.ncep.noaa.gov/pmb/docs/grib2/)
