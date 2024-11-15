pub(crate) mod database;
pub(crate) mod numeric_id;

#[derive(Clone, Debug, derive_more::Display, PartialEq, Eq)]
#[display("({}, {}, {})", abbrev, name, unit)]
pub struct Parameter {
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
    pub fn new(abbrev: &str, name: &str, unit: &str) -> Self {
        Self {
            abbrev: Abbrev(abbrev.to_string()),
            name: name.to_string(),
            unit: unit.to_string(),
        }
    }

    pub fn abbrev(&self) -> &Abbrev {
        &self.abbrev
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, derive_more::Display, Ord, PartialOrd)]
pub struct Abbrev(pub(crate) String);

impl From<&str> for Abbrev {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Abbrev {
    fn from(value: String) -> Self {
        Self(value)
    }
}
