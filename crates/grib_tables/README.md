## grib_tables
Retrieve details of each GRIB parameter from the parameter abbreviation (e.g. "TMP") or from the numeric identifier.

`grib_tables` loads the [GDAL CSV files](https://github.com/OSGeo/gdal/tree/master/frmts/grib/data) into memory and allows the user to:
- Map from parameter abbreviation strings to the numerical representation of that parameter, and the full parameter name and unit.
- Map from the numerical representation of each parameter to the parameter name, abbreviation, and unit.

[Documentation](https://docs.rs/grib_tables/latest/grib_tables/).

## Example

```rust
use grib_tables::{Abbrev, MASTER_TABLE_VERSION, NumericId, Parameter, ParameterDatabase};
# fn main() -> anyhow::Result<()> {

// Get a ParameterDatabase populated with the GRIB tables stored in the included CSV files:
let param_db = ParameterDatabase::new().populate()?;

// Get the numeric IDs and params associated with the abbreviation "TMP":
let abbrev = Abbrev::from("TMP");
let params: Vec<(&NumericId, &Parameter)> = param_db.abbrev_to_parameter(&abbrev);
// `params` is a `Vec` because some abbreviations are associated with multiple parameters.
// (The type of `params` can be deduced by the compiler. The type is written out in
// this example to make it easier to follow the documentation!)

// "TMP" is associated with exactly one parameter:
assert_eq!(params.len(), 1);

// Let's get the `&NumericId` and `&Parameter` associated with the "TMP" abbreviation:
let (temperature_numeric_id, temperature_param) = params.first().as_ref().unwrap();

// Let's get the `name` and `unit` of the `Parameter`:
assert_eq!(temperature_param.name(), "Temperature");
assert_eq!(temperature_param.unit(), "K");

// Let's investigate the `NumericId` associated with "TMP":
assert_eq!(temperature_numeric_id.product_discipline(), 0);
assert_eq!(temperature_numeric_id.parameter_category(), 0);
assert_eq!(temperature_numeric_id.parameter_number(), 0);
assert_eq!(
    temperature_numeric_id.master_table_version(),
    MASTER_TABLE_VERSION
);

// `MAX` values indicate missing values in the GRIB spec.
// "TMP" is part of the GRIB master tables, and so is not
// from a local originating center:
assert_eq!(temperature_numeric_id.originating_center(), u16::MAX);
assert_eq!(temperature_numeric_id.subcenter(), u8::MAX);
assert_eq!(temperature_numeric_id.local_table_version(), u8::MAX);
# Ok(())
# }
```

## Why does `grib_tables` exist?
To build [`hypergrib`](https://github.com/jackkelly/hypergrib), we need to be able to decode GRIB `.idx` files.

We're aware of two awesome existing GRIB readers implemented in Rust ([`gribberish`](https://crates.io/crates/gribberish) and [`grib-rs`](https://crates.io/crates/grib)) but, at the time of writing, neither can decode `.idx` files.

Hence `grib_tables` exists to enable `hypergrib` to decode GRIB `.idx` files.

## Related

- [GRIB tables represented as `.csv` files in GDAL](https://github.com/OSGeo/gdal/tree/master/frmts/grib/data). See [the README for that directory](https://github.com/OSGeo/gdal/blob/master/frmts/grib/degrib/README.TXT).
- [Gribberish discussion](https://github.com/mpiannucci/gribberish/issues/41#issuecomment-2404916278).
- [Post to the gdal-dev mailing list](https://lists.osgeo.org/pipermail/gdal-dev/2024-October/059612.html) about splitting the CSVs and/or copying the CSVs.) 
- [A previous design sketch](https://github.com/JackKelly/rust-playground/tree/main/grib_tables)
  where we try to faithfully capture the GRIB tables hierarchy (Discipline > Category > Parameter).
  But this feels overkill. It might be _slightly_ faster because we can create a perfect hash at compile time.
  But we only need to look things up in the GRIB table whilst indexing a new GRIB dataset.
  When "normal users" use the dataset, they can just read the metadata that we create for the dataset.
