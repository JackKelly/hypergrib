# hypergrib
Lazily read petabytes of GRIBs from cloud object storage.

The ultimate aim is very much inspired by [kerchunk]([url](https://fsspec.github.io/kerchunk/)), [VirtualiZarr]([url](https://github.com/zarr-developers/VirtualiZarr)), and [dynamical.org](https://dynamical.org): Opening a multi-petabyte GRIB dataset from cloud object storage should be as simple as:

```python
dataset = xarray.open_dataset(URL)
```

`hypergrib` is focused on performance: If you're using a VM that's physically close to the cloud object storage, and the VM has a 200 Gbps NIC, then you should be able to read GRIBs at ~20GBytes per second. And each load should incur minimal latency.

The ultimate dream is to be able to train large machine learning models directly from GRIBs on cloud object storage.
