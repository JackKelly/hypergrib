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
