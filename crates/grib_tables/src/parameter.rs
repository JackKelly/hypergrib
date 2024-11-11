use crate::csv_reader::GdalTable4_2Record;

pub(crate) mod database;
pub(crate) mod numeric_id;

#[derive(Clone, Debug, derive_more::Display, PartialEq, Eq)]
#[display("({}, {}, {})", abbrev, name, unit)]
pub(crate) struct Parameter {
    /// Alternative names:
    /// - "short_name" is the column name for "abbrev" in the GDAL CSV files.
    pub(crate) abbrev: Abbrev,

    /// Alternative names:
    /// - MeaningParameterDescription_en: In the wmo-im/GRIB2 CSV files.
    /// - parameter: In the NCEP HTML pages and the WMO PDF.
    pub(crate) name: String,
    pub(crate) unit: String, // TODO: Maybe use a Unit enum?
}

impl Parameter {
    pub(crate) fn new(abbrev: &str, name: &str, unit: &str) -> Self {
        Self {
            abbrev: Abbrev(abbrev.to_string()),
            name: name.to_string(),
            unit: unit.to_string(),
        }
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

#[derive(Hash, Eq, PartialEq, Clone, Debug, derive_more::Display)]
pub(crate) struct Abbrev(pub(crate) String);
