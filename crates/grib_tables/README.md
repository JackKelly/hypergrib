## grib_tables

The basic idea is to keep it simple.

When we're indexing a GRIB dataset with `hypergrib`, `grib_tables` will:
- Load the WMO and GDAL CSVs into memory as a `Vec<Parameter>` with two `HashMap`s: one keyed on
the numerical identifier, and one keyed on the abbreviation string. (Although, actually, maybe
we only need to create one hashmap because, if a dataset has .idx files, then we know we only
need the hashmap keyed on the abbreviation string... although maybe we'll want to also load some
GRIB files.)
- Maybe give users the option to only load some of the GRIB table CSVs. e.g. we don't need the Canadian local
tables if we're loading a NOAA dataset. But this might be over complicating things for a small
reduction in memory footprint.
- Save the decoded metadata into the [JSON that we create for each dataset](https://github.com/JackKelly/hypergrib/discussions/17). Maybe have a mapping
from the abbreviation string to the full ProductTemplate variant.

Then, when users are reading the dataset, we don't need to load any of the GRIB tables because
the relevant metadata will already be captured.

## Why crate does this exist?

Because part of the long-term vision for `hypergrib` is to make it easy to compare different NWP datasets, and to easily gain a detailed understanding of exactly what each NWP dataset captures. Luckily, GRIB files contains a tonne of metadata.

`hypergrib` will create most of its coord labels by reading the `.idx` files. So we need to be able to decode the parameter abbreviation strings (e.g. "VTMP") from `.idx` files. And we want users to easily understand exactly what these params "mean", so we need to decode "TMP" to as much detail as possible.

I'm aware of two existing GRIB readers in Rust (`gribberish` and `grib-rs`) but neither can decode `.idx` files, and neither have a full representation of the GRIB tables. Hence this crate existing! And I'm also keen for this crate to be stand-alone so it could be used by `gribberish` and/or `grib-rs` could use `grib_tables` as their GRIB table representation.

## Related

- https://github.com/JackKelly/rust-playground/tree/main/grib_tables: A previous design sketch
  that I made, where we try to faithfully capture the GRIB tables hierarchy (Discipline > Category > Parameter).
  But this feels overkill. It might be _slightly_ faster because we can create a perfect hash at compile time.
  But then I realised that we only need to look things up in the GRIB table whilst indexing a new GRIB dataset.
  When "normal users" use the dataset, they can just read the metadata that we create for the dataset.
