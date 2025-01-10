# hypergrib

> [!WARNING]
> This code is at a very early stage! It won't do anything useful for a while!

This project is inspired by [kerchunk](https://fsspec.github.io/kerchunk/), [VirtualiZarr](https://github.com/zarr-developers/VirtualiZarr), [dynamical.org](https://dynamical.org), [gribberish](https://github.com/mpiannucci/gribberish), [xarray](https://docs.xarray.dev/en/stable/), [NODD](https://www.noaa.gov/nodd), [Camus Energy's work with GRIB files](https://discourse.pangeo.io/t/pangeo-showcase-optimizations-for-kerchunk-aggregation-and-zarr-i-o-at-scale-for-machine-learning/4074/3), and many other great projects!

## Goals
- Allow users to lazily open petabyte-scale [GRIB](https://en.wikipedia.org/wiki/GRIB) datasets from their laptop with a single line of code: `xr.open_dataset`.
- Lazily open a GRIB dataset with _trillions_ of GRIB messages in a fraction of a second, and minimal memory footprint (see https://github.com/JackKelly/hypergrib/discussions/14)
- Create and constantly update metadata for the main public NWP datasets (so users don't have to do this themselves).
- High performance reading of GRIB binary data: low latency and high bandwidth. A virtual machine with a 200 Gbps (gigabit per second) network interface card in the same region as the data should be able to read GRIBs at ~20 gigabytes per second from object storage. Each load should incur minimal latency. Random access should be as fast & efficient as possible.
- Computational efficiency and "mechanical sympathy" with cloud object storage
- Integrate with `xarray`.

## Background
There are tens of petabytes of GRIB datasets in public cloud object stores. Wouldn't it be nice to be able to lazily open these datasets as easily as possible?!

For example, the [NOAA Open Data Dissemination](https://www.noaa.gov/nodd) (NODD) programme has shared 59 petabytes so far (and growing rapidly), and [ECMWF](https://www.ecmwf.int/en/forecasts/datasets/open-data) are also busily sharing the bulk of their forecasts on cloud object storage. 

`hypergrib` is part of a [broader project which aims to make it as easy as possible to train, run, and research energy forecasting systems](https://github.com/JackKelly/lets_make_it_super_easy_to_use_weather_forecast_data).

For more info on the background and motivation for `hypergrib`, please see [this blog post](https://openclimatefix.org/post/lazy-loading-making-it-easier-to-access-vast-datasets-of-weather-satellite-data).

## Planned design
For the planned design, please see [design.md](https://github.com/JackKelly/hypergrib/blob/main/design.md).

## Why does `hypergrib` exist?
At least to start with, `hypergrib` is an experiment (which stands on the shoulders of giants like `gribberish`, `kerchunk`, `Zarr`, `xarray`, `VirtualiZarr` etc.). The question we're asking with this experiment is: How fast can we go if we "cheat" by building a _special-purpose_ tool focused on reading multi-file GRIB datasets from cloud object storage. Let's throw in all the performance tricks we can think of. And let's also bake in a bunch of domain knowledge about GRIBs.

We're explicitly _not_ trying to build a general-purpose tool like the awesome `kerchunk`.

If `hypergrib` is faster than existing approaches, then maybe ideas from `hypergrib` could be merged into existing tools, and `hypergrib` will remain a testing ground rather than a production tool. Or maybe `hypergrib` will mature into a tool that can be used in production.

## Which read-patterns will perform well with `hypergrib`?
Each GRIB message stores a 2D array: a single horizontal plane. For example, a single GRIB message might store the temperature across the entire globe at a particular timestep, and a particular vertical level. If your use-case reads all of (or a large portion of) the geographical scope of an NWP dataset then you should be able to read directly from GRIB datasets (without creating an intermediate dataset).

For example, ML models that consider large geographical areas should be able to take advantage of the fact that each GRIB message is the entire horizontal plain. For example, energy generation or energy demand models that are trained across multiple countries. Or AI-NWP models which use global NWPs as the initialisation of the state of the atmosphere. But it's currently very hard to get energy data from across the entire globe.

Alternatively, perhaps you are only reading a small geographical region (or even a single point). In this case, you can probably also still read directly from GRIBs *if* you don't need bandwidth of multiple gigabytes per second of useful data.

## Some read patterns will never be well-served by reading directly from GRIBs
There are read-patterns which will never be well-served by reading from GRIBs (because of the way the data is structured on disk). For example, reading a long timeseries for a single geographical point will involve reading about one million times more data from disk than you need (assuming each 2D GRIB message is 1,000 x 1,000 pixels). So, even if you sustain 20 gigabytes per second from GRIBs in object storage, you'll only get 20 _kilobytes_ per second of useful data! For these use-cases, the data will almost certainly have to be converted to something like Zarr. (And, hopefully, `hypergrib` will help make the conversion from GRIB to Zarr as efficient as possible).

That said, we're keen to explore ways to slice _into_ each GRIB message... See [`design.md`](https://github.com/JackKelly/hypergrib/blob/main/design.md#slice-into-each-grib-message).

And, to help people obtain high performance even with read-patterns which aren't well-suited to GRIB, we're very interested in helping [create local high-performance caches of GRIB data](https://github.com/JackKelly/lets_make_it_super_easy_to_use_weather_forecast_data?tab=readme-ov-file#caching-grib-data-so-you-still-get-high-performance-for-read-patterns-which-dont-fit-with-gribs-data-layout) (which is perhaps more realistic than slicing _into_ each GRIB message).

## Name
`hypergrib` uses "hyper" in its mathematical sense, like [hypercube](https://en.wikipedia.org/wiki/Hypercube) (an n-dimensional cube). Oh, and it's reminiscent of a very cool record label, too :)

## GRIB2 documentation

1. [WMO Manual on Codes, Volume I.2, 2023 edition](https://library.wmo.int/records/item/35625-manual-on-codes-volume-i-2-international-codes) - See overview diagram of GRIB messages on PDF page 21.
2. [wgrib C source code](https://github.com/NOAA-EMC/NCEPLIBS-grib_util/blob/develop/src/wgrib/wgrib.c)
3. [NCEP WMO GRIB2 Documentation](https://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_doc/)
4. [GRIB2 use at NCEP](https://www.nco.ncep.noaa.gov/pmb/docs/grib2/)
5. [`GDAL's CSV representation of the GRIB tables`](https://github.com/OSGeo/gdal/tree/master/frmts/grib/data). See the [README for that directory](https://github.com/OSGeo/gdal/blob/master/frmts/grib/degrib/README.TXT)
