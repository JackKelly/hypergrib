use std::{fs::File, path::PathBuf, u8};

use csv::DeserializeRecordsIntoIter;

use crate::parameter::{
    numeric_id::{NumericId, NumericIdBuilder},
    Parameter,
};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct GdalTable4_2Record {
    // This needs to be _signed_ because the first few lines of each GDAL CSV contains comments and
    // have negative `subcat` numbers.
    subcat: i16,
    pub(crate) short_name: String,
    pub(crate) name: String,
    pub(crate) unit: String,
    unit_conv: String,
}

pub(crate) struct GdalTable4_2Iter {
    product_discipline: u8,
    parameter_category: u8,
    path: PathBuf,
    deserializer: DeserializeRecordsIntoIter<File, GdalTable4_2Record>,
}

impl GdalTable4_2Iter {
    fn new(product_discipline: u8, parameter_category: u8) -> anyhow::Result<Self> {
        let filename = format!("grib2_table_4_2_{product_discipline}_{parameter_category}.csv");
        let path = csv_path().join(filename);
        let reader = csv::Reader::from_path(&path)?;
        Ok(Self {
            product_discipline,
            parameter_category,
            path,
            deserializer: reader.into_deserialize(),
        })
    }
}

impl Iterator for GdalTable4_2Iter {
    type Item = (NumericId, Parameter);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(result) = self.deserializer.next() {
            let record: GdalTable4_2Record = result.expect(
                format!(
                    "deserialize result into GdalTable4_2Record for file {:?}",
                    self.path
                )
                .as_str(),
            );

            // Check if we need to skip this line
            let lc_name = record.name.to_lowercase();
            if record.subcat < 0 || lc_name.contains("reserved") || lc_name.contains("missing") {
                continue;
            };

            // Process valid line:
            let numeric_id = NumericIdBuilder::new(
                self.product_discipline,
                self.parameter_category,
                record.subcat.try_into().expect("subcat should be a u8"),
            )
            .build();
            let parameter = record.into();
            return Some((numeric_id, parameter));
        }
        None
    }
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
        let iterator = GdalTable4_2Iter::new(0, 0)?;
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
        let iterator = GdalTable4_2Iter::new(0, 191)?;
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
        let iterator = GdalTable4_2Iter::new(10, 0)?;
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
}
