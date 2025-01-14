use anyhow::Context;

use super::csv_path;

use std::path::PathBuf;

use crate::parameter::{Abbrev, Parameter};

use crate::parameter::numeric_id::NumericIdBuilder;

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct GdalTable4_2Record {
    /// `prod` is present in the _local_ GDAL CSVs, but not the _master_ CSVs.
    #[serde(default)]
    pub(crate) prod: Option<u8>,

    /// `cat` is present in the _local_ GDAL CSVs, but not the _master_ CSVs.
    #[serde(default)]
    pub(crate) cat: Option<u8>,

    /// This needs to be _signed_ because the first few lines of each
    /// GDAL CSV contains comments and have negative `subcat` numbers.
    pub(crate) subcat: i16,

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

pub(crate) fn gdal_table_4_2_iterator(
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

/// Beware that this function includes "grib2_table_4_2_local_index.csv".
pub(crate) fn list_gdal_table_4_2_csv_files() -> Result<glob::Paths, glob::PatternError> {
    let path = csv_path().join("grib2_table_4_2_*.csv");
    glob::glob(path.to_str().expect("path to str"))
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
    fn test_gdal_table_4_2_iterator_bad_path() {
        let result = gdal_table_4_2_iterator(&PathBuf::from("foo"));
        assert!(result.is_err());
    }
}
