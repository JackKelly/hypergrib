#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod csv_reader;
mod parameter;

pub use parameter::database::ParameterDatabase;
pub use parameter::numeric_id::{NumericId, NumericIdBuilder};
pub use parameter::{Abbrev, Parameter};

pub const MASTER_TABLE_VERSION: u8 = 30; // from grib2_table_versions.csv
