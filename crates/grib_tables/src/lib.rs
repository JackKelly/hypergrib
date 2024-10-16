use std::collections::HashMap;

// From https://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_doc/grib2_table4-0.shtml
enum ProductTemplate {
    AnalysisOrForecastAtHorizontalLevel(AnalysisOrForecastAtHorizontalLevel),
    IndividualEnsembleForecastAtHorizontalLevel(IndividualEnsembleForecastAtHorizontalLevel),
}

// From https://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_doc/grib2_temp4-0.shtml
struct AnalysisOrForecastAtHorizontalLevel {
    parameter_category: u8,
    parameter_number: u8,
    generating_process: u8,
    background_generating_process_identifier: u8,
    // TODO: Fill in the rest of these fields.
}

// From https://www.nco.ncep.noaa.gov/pmb/docs/grib2/grib2_doc/grib2_temp4-1.shtml
struct IndividualEnsembleForecastAtHorizontalLevel {
    parameter_category: u8,
    parameter_number: u8,
    generating_process: u8,
    background_generating_process_identifier: u8,
    // TODO: Fill in the rest of these fields.
    // TODO: Think about how to reduce duplication between template definitions.
}

//---------------------- PARAMETER DATABASE: --------------------------

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
struct NumericId {
    product_discipline: u8,
    parameter_category: u8,
    parameter_number: u8,
    originating_center: u8,
    local_table_version: u8,
    master_table_version: u8,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct Abbreviation(String);

struct Parameter {
    numeric_id: NumericId,
    description: String,
    note: String,
    unit: String,
    abbreviation: Abbreviation,
    status: Status,
}

enum Status {
    Operational,
    Deprecated,
}

struct ParameterDatabase {
    params: Vec<Parameter>,

    /// Maps from the `NumericId` of the `Parameter` to the index into `params`.
    numeric_lookup: HashMap<NumericId, usize>,

    /// Maps from the `Abbreviation` of the `Parameter` to the index into `params`.
    abbrev_lookup: HashMap<Abbreviation, usize>,
}

enum ParameterInsertionError {
    NumericKeyAlreadyExists,
    AbbrevKeyAlreadyExists,
}

impl ParameterDatabase {
    fn new() -> Self {
        Self {
            params: vec![],
            numeric_lookup: HashMap::new(),
            abbrev_lookup: HashMap::new(),
        }
    }

    fn insert(&mut self, parameter: Parameter) -> Result<(), (Parameter, ParameterInsertionError)> {
        let index = self.params.len();

        // Insert into self.numeric_lookup if the key doesn't exist yet:
        if self.numeric_lookup.contains_key(&parameter.numeric_id) {
            return Err((parameter, ParameterInsertionError::NumericKeyAlreadyExists));
        }
        self.numeric_lookup.insert(parameter.numeric_id, index);

        // Insert into self.numeric_lookup if the key doesn't exist yet:
        if self.abbrev_lookup.contains_key(&parameter.abbreviation) {
            return Err((parameter, ParameterInsertionError::AbbrevKeyAlreadyExists));
        }
        self.abbrev_lookup
            .insert(parameter.abbreviation.clone(), index);

        self.params.push(parameter);
        Ok(())
    }

    fn abbreviation_to_parameter(&self, abbreviation: &Abbreviation) -> Option<&Parameter> {
        let index = self.abbrev_lookup.get(abbreviation)?;
        Some(&self.params[*index])
    }

    fn numeric_id_to_parameter(&self, numeric_id: &NumericId) -> Option<&Parameter> {
        let index = self.numeric_lookup.get(numeric_id)?;
        Some(&self.params[*index])
    }

    fn len(&self) -> usize {
        self.params.len()
    }
}

// The basic idea is to keep it simple.
// When we're indexing a GRIB dataset:
// - Load the WMO and GDAL CSVs into memory as a simple "database" with two HashMaps: one keyed on
// the numerical identifier, and one keyed on the abbreviation string. (Although, actually, maybe
// we only need to create one hashmap because, if a dataset has .idx files, then we know we only
// need the hashmap keyed on the abbreviation string... although maybe we'll want to also load some
// GRIB files.)
// - Maybe give the option to only load some of the CSVs. e.g. we don't need the Canadian local
// tables if we're loading a NOAA dataset. But this might be over complicating things for a small
// reduction in memory footprint.
// - Save the decoded metadata into the JSON that we create for each dataset. Maybe have a mapping
// from the abbreviation string to the full ProductTemplate variant.
//
// Then, when users are reading the dataset, we don't need to load any of the GRIB tables because
// the relevant metadata will already be captured.

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_and_retreive() {
        let numeric_id = NumericId {
            product_discipline: 0,
            parameter_category: 0,
            parameter_number: 0,
            originating_center: 0,
            local_table_version: 0,
            master_table_version: 0,
        };

        let abbreviation = Abbreviation("FOO".to_string());

        let param = Parameter {
            numeric_id,
            description: "Foo".to_string(),
            note: "Bar".to_string(),
            unit: "K".to_string(),
            abbreviation: abbreviation.clone(),
            status: Status::Operational,
        };
    }
}
