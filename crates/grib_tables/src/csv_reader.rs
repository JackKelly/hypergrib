use std::path::PathBuf;

use crate::parameter::{
    numeric_id::{NumericId, NumericIdBuilder},
    Abbrev, Parameter,
};

#[derive(Debug, serde::Deserialize, Clone)]
pub(crate) struct GdalTable4_2Record {
    /// `prod` is present in the _local_ GDAL CSVs, but not the _master_ CSVs.
    #[serde(default)]
    prod: Option<u8>,

    /// `cat` is present in the _local_ GDAL CSVs, but not the _master_ CSVs.
    #[serde(default)]
    cat: Option<u8>,

    /// This needs to be _signed_ because the first few lines of each GDAL CSV contains comments and
    /// have negative `subcat` numbers.
    subcat: i16,

    pub(crate) short_name: String,
    pub(crate) name: String,
    pub(crate) unit: String,
}

impl Into<(NumericId, Parameter)> for GdalTable4_2Record {
    fn into(self) -> (NumericId, Parameter) {
        let numeric_id = NumericIdBuilder::new(
            self.prod.unwrap(),
            self.cat.unwrap(),
            self.subcat.try_into().expect("subcat must be a u8"),
        )
        .build();
        let parameter = self.into();
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

fn gdal_table_4_2_iterator(
    path: &PathBuf,
) -> anyhow::Result<impl Iterator<Item = GdalTable4_2Record>> {
    let reader = csv::Reader::from_path(path)?;
    let deser_error_msg = format!(
        "deserialize result into GdalTable4_2Record for file {:?}",
        path
    );
    let iter = reader
        .into_deserialize()
        .map(move |result| -> GdalTable4_2Record { result.expect(&deser_error_msg) })
        .filter(|record| {
            let lc_name = record.name.to_lowercase();
            record.subcat >= 0 && !lc_name.contains("reserved") && !lc_name.contains("missing")
        });
    Ok(iter)
}

pub(crate) fn gdal_table_4_2_master(
    product_discipline: u8,
    parameter_category: u8,
) -> anyhow::Result<impl Iterator<Item = (NumericId, Parameter)>> {
    let filename = format!("grib2_table_4_2_{product_discipline}_{parameter_category}.csv");
    let path = csv_path().join(filename);
    let iter = gdal_table_4_2_iterator(&path)?;
    Ok(iter.map(move |mut record| {
        record.prod = record.prod.or(Some(product_discipline));
        record.cat = record.cat.or(Some(parameter_category));
        record.into()
    }))
}

fn csv_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    manifest_dir.join("csv")
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_read_gdal_table_4_2_0_0() -> anyhow::Result<()> {
        let iterator = gdal_table_4_2_master(0, 0)?;
        let vec: Vec<_> = iterator.collect();
        assert_eq!(vec.len(), 33);

        // Check first row of data:
        let (numeric_id, parameter) = &vec[0];
        assert_eq!(numeric_id, &NumericIdBuilder::new(0, 0, 0).build());
        assert_eq!(parameter, &Parameter::new("TMP", "Temperature", "K"));

        // Check middle row of data:
        let (numeric_id, parameter) = &vec[16];
        assert_eq!(numeric_id, &NumericIdBuilder::new(0, 0, 16).build());
        assert_eq!(
            parameter,
            &Parameter::new("SNOHF", "Snow phase change heat flux", "W/m^2")
        );

        // Check last row of data:
        let (numeric_id, parameter) = &vec[32];
        assert_eq!(numeric_id, &NumericIdBuilder::new(0, 0, 32).build());
        assert_eq!(
            parameter,
            &Parameter::new("", "Wet-bulb potential temperature", "K")
        );
        Ok(())
    }

    #[test]
    fn test_read_gdal_table_4_2_0_191() -> anyhow::Result<()> {
        let iterator = gdal_table_4_2_master(0, 191)?;
        let vec: Vec<_> = iterator.collect();
        assert_eq!(vec.len(), 4);

        // Check first row of data:
        let (numeric_id, parameter) = &vec[0];
        assert_eq!(numeric_id, &NumericIdBuilder::new(0, 191, 0).build());
        assert_eq!(
            parameter,
            &Parameter::new(
                "TSEC",
                "Seconds prior to initial reference time (defined in Section 1)",
                "s"
            )
        );

        // Check last row of data:
        let (numeric_id, parameter) = &vec[3];
        assert_eq!(numeric_id, &NumericIdBuilder::new(0, 191, 3).build());
        assert_eq!(
            parameter,
            &Parameter::new("DSLOBS", "Days Since Last Observation", "d")
        );
        Ok(())
    }

    #[test]
    fn test_read_gdal_table_4_2_10_0() -> anyhow::Result<()> {
        let iterator = gdal_table_4_2_master(10, 0)?;
        let vec: Vec<_> = iterator.collect();
        assert_eq!(vec.len(), 74);

        // Check first row of data:
        let (numeric_id, parameter) = &vec[0];
        assert_eq!(numeric_id, &NumericIdBuilder::new(10, 0, 0).build());
        assert_eq!(parameter, &Parameter::new("WVSP1", "Wave spectra (1)", "-"));

        // Check last row of data:
        let (numeric_id, parameter) = &vec[73];
        assert_eq!(numeric_id, &NumericIdBuilder::new(10, 0, 73).build());
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
            .map(|record| -> (NumericId, Parameter) { record.into() })
            .collect();
        assert_eq!(vec.len(), 391);

        // Check first row of data:
        let (numeric_id, parameter) = &vec[0];
        assert_eq!(numeric_id, &NumericIdBuilder::new(0, 0, 192).build());
        assert_eq!(
            parameter,
            &Parameter::new("SNOHF", "Snow Phase Change Heat Flux", "W/(m^2)")
        );

        Ok(())
    }
}
