use core::panic;
use std::{collections::HashMap, num::IntErrorKind, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Deserializer};

use crate::{parameter::{
    database::ParameterDatabase,
    numeric_id::{NumericId, NumericIdBuilder},
    Abbrev, Parameter,
}, MASTER_TABLE_VERSION};

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct GdalTable4_2Record {
    /// `prod` is present in the _local_ GDAL CSVs, but not the _master_ CSVs.
    #[serde(default)]
    prod: Option<u8>,

    /// `cat` is present in the _local_ GDAL CSVs, but not the _master_ CSVs.
    #[serde(default)]
    cat: Option<u8>,

    /// This needs to be _signed_ because the first few lines of each
    /// GDAL CSV contains comments and have negative `subcat` numbers.
    subcat: i16,

    pub(crate) short_name: String,
    pub(crate) name: String,
    pub(crate) unit: String,
}

impl From<GdalTable4_2Record> for (NumericIdBuilder, Parameter) {
    fn from(record: GdalTable4_2Record) -> Self {
        let numeric_id = (&record).into();
        let parameter = record.into();
        (numeric_id, parameter)
    }
}

impl From<GdalTable4_2Record> for Parameter {
    fn from(record: GdalTable4_2Record) -> Self {
        Self {
            abbrev: Abbrev(record.short_name),
            name: record.name,
            unit: record.unit,
        }
    }
}

impl From<&GdalTable4_2Record> for NumericIdBuilder {
    fn from(record: &GdalTable4_2Record) -> Self {
        NumericIdBuilder::new(
            record.prod.unwrap(),
            record.cat.unwrap(),
            record.subcat.try_into().expect("subcat must be a u8"),
        )
    }
}

#[derive(Debug, serde::Deserialize)]
struct GdalLocalIndex {
    center_code: u16,

    #[serde(deserialize_with = "deserialize_subcenter_code")]
    subcenter_code: u8,

    filename: String,
}


fn deserialize_subcenter_code<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer).unwrap();
    let result: Result<u16, _> = s.parse();
    match result {
        Ok(value) => {
            match value {
                u16::MAX => Ok(u8::MAX),
                v => v
                    .try_into()
                    .with_context(|| format!("Failed to convert {value:?} into a u8 subcenter code!"))
                    .map_err(serde::de::Error::custom),
            }
        },
        Err(err) if *err.kind() == IntErrorKind::Empty => Ok(u8::MAX),
        Err(err) => Err(serde::de::Error::custom(err))

    }
}

/// The values of the HashMap are the (center_code, subcenter_code).
fn get_local_index() -> HashMap<String, (u16, u8)> {
    let path = csv_path().join("grib2_table_4_2_local_index.csv");
    let mut reader = csv::Reader::from_path(&path).with_context(|| format!("Failed: csv::Reader::from_path({path:?})")).unwrap();
    let mut map = HashMap::new();
    for row in reader.deserialize() {
        let record: GdalLocalIndex = row.with_context(|| format!("Failed to deserialize row from {path:?}")).unwrap();

        // Skip duplicate:
        if map.contains_key(&record.filename) 
            && record.filename == "grib2_table_4_2_local_NDFD.csv" 
            && record.center_code == 8 
            && record.subcenter_code == u8::MAX {
            continue;
        }

        let center_and_subcenter_codes = (record.center_code, record.subcenter_code);
        match map.insert(record.filename, center_and_subcenter_codes) {
            None => (),
            Some(old_value) => panic!(
                "{path:?} contains duplicate filenames! Old center_code={}, old subcenter_code={}", 
                old_value.0, old_value.1),
        }
    }
    map
}

fn gdal_table_4_2_iterator(
    path: &PathBuf,
) -> anyhow::Result<impl Iterator<Item = GdalTable4_2Record>> {
    let reader = csv::Reader::from_path(path)
        .with_context(|| format!("Error when calling csv::Reader::from_path({path:?})"))?;
    let deser_error_msg = format!("deserialize result into GdalTable4_2Record for path {path:?}",);
    let iter = reader
        .into_deserialize()
        .map(move |row| -> GdalTable4_2Record { row.expect(&deser_error_msg) })
        .filter(|record| {
            let lc_name = record.name.to_lowercase();
            record.subcat >= 0 && !lc_name.contains("reserved") && !lc_name.contains("missing")
        });
    Ok(iter)
}

pub(crate) fn gdal_master_table_4_2_iterator(
    product_discipline: u8,
    parameter_category: u8,
) -> anyhow::Result<impl Iterator<Item = (NumericIdBuilder, Parameter)>> {
    let filename = format!("grib2_table_4_2_{product_discipline}_{parameter_category}.csv");
    let path = csv_path().join(filename);
    let iter = gdal_table_4_2_iterator(&path)?;
    Ok(iter.map(move |mut record| {
        record.prod = Some(product_discipline);
        record.cat = Some(parameter_category);
        record.into()
    }))
}

fn csv_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    manifest_dir.join("csv")
}

/// Beware that this function includes "grib2_table_4_2_local_index.csv".
fn list_gdal_table_4_2_csv_files() -> Result<glob::Paths, glob::PatternError> {
    let path = csv_path().join("grib2_table_4_2_*.csv");
    glob::glob(path.to_str().expect("path to str"))
}

fn get_populated_param_database() -> anyhow::Result<ParameterDatabase> {
    let mut param_db = ParameterDatabase::new();
    let local_index = get_local_index();

    let re_master_table =
        regex::Regex::new(r"^grib2_table_4_2_(?<discipline>\d{1,2})_(?<category>\d{1,3}).csv$")
            .unwrap();
    let re_local_table = regex::Regex::new(r"^grib2_table_4_2_local_[A-Z][A-Za-z]+.csv$").unwrap();
    for path in list_gdal_table_4_2_csv_files()? {
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
            for record in gdal_master_table_4_2_iterator(discipline, category)? {
                let (mut numeric_id_builder, parameter) = record;
                numeric_id_builder.set_master_table_version(MASTER_TABLE_VERSION);
                let numeric_id = numeric_id_builder.build();
                param_db.insert(numeric_id, parameter).with_context(|| 
                    format!("Error when inserting into parameter database. Master table 4.2 path={path:?}")
                )?;
            }
        } else if re_local_table.is_match(file_name) {
            for record in gdal_table_4_2_iterator(&path)? {
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

    use super::*;

    #[test]
    fn test_read_gdal_table_4_2_0_0() -> anyhow::Result<()> {
        let iterator = gdal_master_table_4_2_iterator(0, 0)?;
        let vec: Vec<_> = iterator.collect();
        assert_eq!(vec.len(), 33);

        // Check first row of data:
        let (numeric_id_builder, parameter) = &vec[0];
        assert_eq!(numeric_id_builder, &NumericIdBuilder::new(0, 0, 0));
        assert_eq!(parameter, &Parameter::new("TMP", "Temperature", "K"));

        // Check middle row of data:
        let (numeric_id_builder, parameter) = &vec[16];
        assert_eq!(numeric_id_builder, &NumericIdBuilder::new(0, 0, 16));
        assert_eq!(
            parameter,
            &Parameter::new("SNOHF", "Snow phase change heat flux", "W/m^2")
        );

        // Check last row of data:
        let (numeric_id_builder, parameter) = &vec[32];
        assert_eq!(numeric_id_builder, &NumericIdBuilder::new(0, 0, 32));
        assert_eq!(
            parameter,
            &Parameter::new("", "Wet-bulb potential temperature", "K")
        );
        Ok(())
    }

    #[test]
    fn test_read_gdal_table_4_2_0_191() -> anyhow::Result<()> {
        let iterator = gdal_master_table_4_2_iterator(0, 191)?;
        let vec: Vec<_> = iterator.collect();
        assert_eq!(vec.len(), 4);

        // Check first row of data:
        let (numeric_id_builder, parameter) = &vec[0];
        assert_eq!(numeric_id_builder, &NumericIdBuilder::new(0, 191, 0));
        assert_eq!(
            parameter,
            &Parameter::new(
                "TSEC",
                "Seconds prior to initial reference time (defined in Section 1)",
                "s"
            )
        );

        // Check last row of data:
        let (numeric_id_builder, parameter) = &vec[3];
        assert_eq!(numeric_id_builder, &NumericIdBuilder::new(0, 191, 3));
        assert_eq!(
            parameter,
            &Parameter::new("DSLOBS", "Days Since Last Observation", "d")
        );
        Ok(())
    }

    #[test]
    fn test_read_gdal_table_4_2_10_0() -> anyhow::Result<()> {
        let iterator = gdal_master_table_4_2_iterator(10, 0)?;
        let vec: Vec<_> = iterator.collect();
        assert_eq!(vec.len(), 74);

        // Check first row of data:
        let (numeric_id_builder, parameter) = &vec[0];
        assert_eq!(numeric_id_builder, &NumericIdBuilder::new(10, 0, 0));
        assert_eq!(parameter, &Parameter::new("WVSP1", "Wave spectra (1)", "-"));

        // Check last row of data:
        let (numeric_id_builder, parameter) = &vec[73];
        assert_eq!(numeric_id_builder, &NumericIdBuilder::new(10, 0, 73));
        assert_eq!(
            parameter,
            &Parameter::new("", "Whitecap fraction", "fraction")
        );
        Ok(())
    }

    #[test]
    fn test_read_gdal_table_4_2_local_NCEP() -> anyhow::Result<()> {
        let path = csv_path().join("grib2_table_4_2_local_NCEP.csv");
        let iterator = gdal_table_4_2_iterator(&path)?;
        let vec: Vec<_> = iterator
            .map(|record| -> (NumericIdBuilder, Parameter) { record.into() })
            .collect();
        assert_eq!(vec.len(), 391);

        // Check first row of data:
        let (numeric_id_builder, parameter) = &vec[0];
        assert_eq!(numeric_id_builder, &NumericIdBuilder::new(0, 0, 192));
        assert_eq!(
            parameter,
            &Parameter::new("SNOHF", "Snow Phase Change Heat Flux", "W/(m^2)")
        );

        Ok(())
    }

    #[test]
    fn test_list_gdal_master_table_4_2_csv_files() -> anyhow::Result<()> {
        let filenames: Vec<_> = list_gdal_table_4_2_csv_files()?.collect();
        assert_eq!(filenames.len(), 60);
        Ok(())
    }

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

        // TODO: Do something with database!
        Ok(())
    }

    #[test]
    fn test_gdal_table_4_2_iterator_bad_path() {
        let result = gdal_table_4_2_iterator(&PathBuf::from("foo"));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_local_index() {
        let local_index = get_local_index();
        assert_eq!(local_index.len(), 5);
    }
}
