use anyhow::Context;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// This is a u64 because `NumericId` is used as the key in a `BTreeMap`, and u64s are very fast to
/// compare. (And `BTreeMaps` frequently compare keys!)
#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone, Debug, derive_more::Display)]
struct NumericId(u64);

impl NumericId {
    /// `originating_center` and `local_table_version` must be `u16::MAX` and `u8::MAX` respectively for parameters
    /// which belong to the master table.
    fn new(
        product_discipline: u8,
        parameter_category: u8,
        parameter_number: u8,
        master_table_version: u8,
        originating_center: u16,
        local_table_version: u8,
    ) -> Self {
        const BITS_PER_BYTE: u64 = 8;
        let mut numeric_id = 0;
        numeric_id &= (product_discipline as u64) << BITS_PER_BYTE * 6;
        numeric_id &= (parameter_category as u64) << BITS_PER_BYTE * 5;
        numeric_id &= (parameter_number as u64) << BITS_PER_BYTE * 4;
        numeric_id &= (master_table_version as u64) << BITS_PER_BYTE * 3;
        numeric_id &= (originating_center as u64) << BITS_PER_BYTE * 1;
        numeric_id &= local_table_version as u64;
        Self(numeric_id)
    }

    // TODO: Test this!
    fn discipline(&self) -> u8 {
        (self.0 & 0x00_FF_00_00_00_00_00_00).try_into().unwrap()
    }

    // TODO: Implement getters for the other numeric identifiers. And test!
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, derive_more::Display)]
struct Abbreviation(String);

#[derive(Clone, Debug, derive_more::Display, PartialEq, Eq)]
#[display(
    "({}, {}, {}, {}, {}, {})",
    numeric_id,
    description,
    note,
    unit,
    abbreviation,
    status
)]
struct Parameter {
    description: String,
    note: String,
    unit: String, // TODO: Maybe use a Unit enum?
    abbreviation: Abbreviation,
    status: Status,
}

#[derive(Clone, Debug, derive_more::Display, PartialEq, Eq)]
enum Status {
    Operational,
    Deprecated,
}

struct ParameterDatabase {
    /// We use a `BTreeMap` so we can get, say, all the versions of a particular parameter_number
    /// using BTreeMap.range.
    numeric_id_to_params: BTreeMap<NumericId, Parameter>,

    /// TODO: Empirically test if we actually need the value to be a BTreeSet (instead of just a
    /// NumericId). In other words, check if any GRIB abbreviations map to multiple parameters.
    abbrev_to_numeric_id: HashMap<Abbreviation, BTreeSet<NumericId>>,
}

#[derive(thiserror::Error, Debug, derive_more::Display)]
enum ParameterInsertionError {
    NumericKeyAlreadyExists(Parameter),
}

enum AbbrevToParameter<'a> {
    Unique((&'a NumericId, &'a Parameter)),
    Multiple(Vec<(&'a NumericId, &'a Parameter)>),
    AbbrevNotFound,
}

impl ParameterDatabase {
    fn new() -> Self {
        Self {
            numeric_id_to_params: BTreeMap::new(),
            abbrev_to_numeric_id: HashMap::new(),
        }
    }

    fn insert(
        &mut self,
        numeric_id: NumericId,
        parameter: Parameter,
    ) -> Result<(), ParameterInsertionError> {
        self.abbrev_to_numeric_id
            .entry(parameter.abbreviation.clone())
            .and_modify(|set| {
                set.insert(numeric_id);
            })
            .or_insert(BTreeSet::from([numeric_id]));

        if self.numeric_id_to_params.contains_key(&numeric_id) {
            return Err(ParameterInsertionError::NumericKeyAlreadyExists(parameter));
        }
        self.numeric_id_to_params
            .insert(numeric_id, parameter)
            .unwrap();
        Ok(())
    }

    fn abbreviation_to_parameter(&self, abbreviation: &Abbreviation) -> AbbrevToParameter {
        let numeric_ids = match self.abbrev_to_numeric_id.get(abbreviation) {
            None => return AbbrevToParameter::AbbrevNotFound,
            Some(numeric_ids) => numeric_ids,
        };
        if numeric_ids.len() == 1 {
            let numeric_id = numeric_ids.first().unwrap();
            let param = self.numeric_id_to_params.get(&numeric_id).unwrap();
            AbbrevToParameter::Unique((numeric_id, param))
        } else {
            AbbrevToParameter::Multiple(
                numeric_ids
                    .iter()
                    .map(|numeric_id| {
                        let param = self.numeric_id_to_params.get(numeric_id).unwrap();
                        (numeric_id, param)
                    })
                    .collect(),
            )
        }
    }

    fn numeric_id_to_parameter(&self, numeric_id: &NumericId) -> Option<&Parameter> {
        self.numeric_id_to_params.get(numeric_id)
    }

    fn len(&self) -> usize {
        self.numeric_id_to_params.len()
    }
}

#[cfg(test)]
mod test {
    use anyhow::Context;

    use super::*;

    #[test]
    fn insert_and_retreive() -> anyhow::Result<()> {
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

        let mut param_db = ParameterDatabase::new();
        assert_eq!(param_db.len(), 0);

        param_db.insert(param.clone())?;
        assert_eq!(param_db.len(), 1);

        let retrieved_param = param_db
            .abbreviation_to_parameter(&param.abbreviation)
            .context("Failed to map from abbrev to param")?;

        assert_eq!(param, *retrieved_param);

        Ok(())
    }
}
