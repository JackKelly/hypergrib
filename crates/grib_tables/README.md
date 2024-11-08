## grib_tables

`grib_tables` will:
- Load the WMO and GDAL CSVs into memory as a `ParameterDatabase`.
- Maybe give users the option to only load some of the GRIB table CSVs. e.g. we don't need the Canadian local
tables if we're loading a NOAA dataset. But this might be over complicating things for a small
reduction in memory footprint.
- Save the decoded metadata into the [JSON that we create for each dataset](https://github.com/JackKelly/hypergrib/discussions/17). Maybe have a mapping
from the abbreviation string to the full ProductTemplate variant.

Then, when users reading the dataset using `hypergrib`, we don't need to load any GRIB tables because
the relevant metadata will already be captured.

## Why crate does this exist?

Because part of the long-term vision for `hypergrib` is to make it easy to compare different NWP datasets, and to easily gain a detailed understanding of exactly what each NWP dataset captures. Luckily, GRIB files contains a tonne of metadata.

`hypergrib` will create most of its coord labels by reading the `.idx` files. So we need to be able to decode the parameter abbreviation strings (e.g. "VTMP") from `.idx` files. And we want users to easily understand exactly what these params "mean", so we need to decode "TMP" to as much detail as possible.

I'm aware of two existing GRIB readers in Rust (`gribberish` and `grib-rs`) but neither can decode `.idx` files, and neither have a full representation of the GRIB tables. Hence this crate existing! And I'm also keen for this crate to be stand-alone so it could be used by `gribberish` and/or `grib-rs` could use `grib_tables` as their GRIB table representation.

## Cloning this repo

This module uses [git submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules) to keep the relevant CSV files in this repo (inspired by `grib-rs`).

When you clone this repo, pass the `--recurse-submodules` option to `git clone`.

If you've already cloned this repo then run `git submodule update --init --recursive`

## Related

- https://github.com/JackKelly/rust-playground/tree/main/grib_tables: A previous design sketch
  that I made, where we try to faithfully capture the GRIB tables hierarchy (Discipline > Category > Parameter).
  But this feels overkill. It might be _slightly_ faster because we can create a perfect hash at compile time.
  But then I realised that we only need to look things up in the GRIB table whilst indexing a new GRIB dataset.
  When "normal users" use the dataset, they can just read the metadata that we create for the dataset.
