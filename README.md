# hypergrib
Lazily read petabytes of GRIBs from cloud object storage.

The ultimate aim is very much inspired by [kerchunk](https://fsspec.github.io/kerchunk/), [VirtualiZarr](https://github.com/zarr-developers/VirtualiZarr), and [dynamical.org](https://dynamical.org): Opening a multi-petabyte GRIB dataset from cloud object storage should be as simple as:

```python
dataset = xarray.open_dataset(URL)
```

`hypergrib` is focused on performance: If you're using a VM with a 200 Gbps NIC close to the cloud object storage then you should be able to read GRIBs at ~20 GBytes per second. And each load should incur minimal latency. `hypergrib` will use all the tricks from the [AnyBlob paper](https://www.vldb.org/pvldb/vol16/p2769-durner.pdf) to achieve max performance.

The ultimate dream is to be able to train large machine learning models directly from GRIBs on cloud object storage.
