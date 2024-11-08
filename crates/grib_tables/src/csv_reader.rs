use std::path::PathBuf;

use crate::parameter::{database::ParameterDatabase, Parameter};

struct GdalTable_4_2_Record {
    subcat: u8,
    short_name: String,
    name: String,
    unit: String,
    unit_conv: String,
}

fn read_table_4_2(
    parameter_db: &mut ParameterDatabase,
    discipline: u8,
    category: u8,
) -> anyhow::Result<()> {
    let filename = format!("grib2_table_4_2_{discipline}_{category}.csv");
    let path = csv_path().join(filename);
    let mut reader = csv::Reader::from_path(path);
    for result in reader.deserialize() {
        let record: Parameter = result?;
    }
    Ok(())
}

fn csv_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    manifest_dir.join("csv")
}
