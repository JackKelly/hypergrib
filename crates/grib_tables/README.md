## grib_tables

`grib_tables` loads the GDAL CSVs into memory and allow the user to:
- Map from parameter abbreviation strings to the numerical representation of that parameter, and the full param name and unit.
- Map from the numerical representation of each parameter to the param name, abbreviation, and unit.

## Example

```rust
use grib_tables::{ParameterDatabase, MASTER_TABLE_VERSION, Abbrev};
# fn main() -> anyhow::Result<()> {
// Get a ParameterDatabase populated with the GRIB tables stored in the included CSV files:
let param_db = ParameterDatabase::new().populate()?;

// Get the numeric IDs and params associated with the abbreviation "TMP":
let params = param_db.abbrev_to_parameter(&Abbrev::from("TMP"));

// `params` is a `Vec` because some abbreviations are associated with multiple parameters.
// However, "TMP" is associated with exactly one parameter:
assert_eq!(params.len(), 1);
let (temperature_numeric_id, temperature_param) = params.first().as_ref().unwrap();
assert_eq!(temperature_param.name(), "Temperature");
assert_eq!(temperature_param.unit(), "K");
assert_eq!(temperature_numeric_id.product_discipline(), 0);
assert_eq!(temperature_numeric_id.parameter_category(), 0);
assert_eq!(temperature_numeric_id.parameter_number(), 0);
assert_eq!(
    temperature_numeric_id.master_table_version(),
    MASTER_TABLE_VERSION
);
assert_eq!(temperature_numeric_id.originating_center(), u16::MAX);
assert_eq!(temperature_numeric_id.subcenter(), u8::MAX);
assert_eq!(temperature_numeric_id.local_table_version(), u8::MAX);
# Ok(())
# }
```

## Why crate does this exist?

Because part of the long-term vision for `hypergrib` is to make it easy to compare different NWP datasets, and to easily gain a detailed understanding of exactly what each NWP dataset captures. Luckily, GRIB files contains a tonne of metadata.

`hypergrib` will create most of its coord labels by reading the `.idx` files. So we need to be able to decode the parameter abbreviation strings (e.g. "VTMP") from `.idx` files. And we want users to easily understand exactly what these params "mean", so we need to decode "TMP" to as much detail as possible.

I'm aware of two existing GRIB readers in Rust (`gribberish` and `grib-rs`) but neither can decode `.idx` files, and neither have a full representation of the GRIB tables. Hence this crate existing! And I'm also keen for this crate to be stand-alone so it could be used by `gribberish` and/or `grib-rs` could use `grib_tables` as their GRIB table representation.

## Related

- [GRIB tables represented as `.csv` files in GDAL](https://github.com/OSGeo/gdal/tree/master/frmts/grib/data). See [the README for that directory](https://github.com/OSGeo/gdal/blob/master/frmts/grib/degrib/README.TXT)
- [Gribberish discussion](https://github.com/mpiannucci/gribberish/issues/41#issuecomment-2404916278)
- [My post to the gdal-dev mailing list](https://lists.osgeo.org/pipermail/gdal-dev/2024-October/059612.html) about splitting the CSVs and/or me copying the CSVs.) 
- https://github.com/JackKelly/rust-playground/tree/main/grib_tables: A previous design sketch
  that I made, where we try to faithfully capture the GRIB tables hierarchy (Discipline > Category > Parameter).
  But this feels overkill. It might be _slightly_ faster because we can create a perfect hash at compile time.
  But then I realised that we only need to look things up in the GRIB table whilst indexing a new GRIB dataset.
  When "normal users" use the dataset, they can just read the metadata that we create for the dataset.
