mod read_local_index;
mod read_table_4_2;

use std::path::PathBuf;

use anyhow::Context;

use crate::{parameter::database::ParameterDatabase, MASTER_TABLE_VERSION};


fn csv_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    manifest_dir.join("csv")
}

fn get_populated_param_database() -> anyhow::Result<ParameterDatabase> {
    let mut param_db = ParameterDatabase::new();
    let local_index = read_local_index::get_local_index();

    let re_master_table =
        regex::Regex::new(r"^grib2_table_4_2_(?<discipline>\d{1,2})_(?<category>\d{1,3}).csv$")
            .unwrap();
    let re_local_table = regex::Regex::new(r"^grib2_table_4_2_local_[A-Z][A-Za-z]+.csv$").unwrap();
    for path in read_table_4_2::list_gdal_table_4_2_csv_files()? {
        let path = path?;
        let file_name = path
            .file_name()
            .with_context(|| format!("Failed to get file_name from path {path:?}"))?
            .to_str()
            .with_context(|| format!("Failed to convert file_stem to &str for path {path:?}"))?;
        if file_name == "grib2_table_4_2_local_index.csv" {
            continue;
        } else if let Some(captures) = re_master_table.captures(file_name) {
            let discipline = (&captures["discipline"]).parse().expect("parse discipline");
            let category = (&captures["category"]).parse().expect("parse category");
            for record in read_table_4_2::gdal_master_table_4_2_iterator(discipline, category)? {
                let (mut numeric_id_builder, parameter) = record;
                numeric_id_builder.set_master_table_version(MASTER_TABLE_VERSION);
                let numeric_id = numeric_id_builder.build();
                param_db.insert(numeric_id, parameter).with_context(|| 
                    format!("Error when inserting into parameter database. Master table 4.2 path={path:?}")
                )?;
            }
        } else if re_local_table.is_match(file_name) {
            for record in read_table_4_2::gdal_table_4_2_iterator(&path)? {
                let (mut numeric_id_builder, parameter) = record.into();
                numeric_id_builder.set_master_table_version(MASTER_TABLE_VERSION);
                let (originating_center, subcenter) = local_index[file_name];
                numeric_id_builder.set_originating_center(originating_center);
                numeric_id_builder.set_subcenter(subcenter);
                let numeric_id = numeric_id_builder.build();
                param_db.insert(numeric_id, parameter).with_context(||
                    format!("Error when inserting into parameter database. Local table 4.2 path={path:?}")
                )?;
            }
        } else {
            return Err(anyhow::format_err!("Failed to interpret CSV path {path:?}!"));
        }
    }
    Ok(param_db)
}

#[cfg(test)]
mod test {

    use crate::parameter::Abbrev;

    use super::*;
    #[test]
    fn test_get_populated_param_database() -> anyhow::Result<()> {
        let param_db = get_populated_param_database()?;
        println!("length of param db = {}", param_db.len());
        assert_eq!(param_db.len(), 1669);

        // Check Temperature
        let params = param_db.abbrev_to_parameter(&Abbrev("TMP".to_string()));
        assert_eq!(params.len(), 1);
        let (temperature_numeric_id, temperature_param) = params.first().as_ref().unwrap();
        assert_eq!(temperature_param.name, "Temperature");
        assert_eq!(temperature_param.unit, "K");
        assert_eq!(temperature_numeric_id.product_discipline(), 0);
        assert_eq!(temperature_numeric_id.parameter_category(), 0);
        assert_eq!(temperature_numeric_id.parameter_number(), 0);
        assert_eq!(temperature_numeric_id.master_table_version(), MASTER_TABLE_VERSION);
        assert_eq!(temperature_numeric_id.originating_center(), u16::MAX);
        assert_eq!(temperature_numeric_id.subcenter(), u8::MAX);
        assert_eq!(temperature_numeric_id.local_table_version(), u8::MAX);

        // TODO: Test some local parameters.
        Ok(())
    }

    #[test]
    fn test_for_duplicate_abbreviations() -> anyhow::Result<()> {
        let param_db = get_populated_param_database()?;
        println!("{}", param_db.describe_all_duplicate_abbrevs());
        Ok(())
    }
}
