use std::collections::BTreeSet;

use std::collections::HashMap;

use super::{numeric_id::NumericId, Abbreviation, Parameter};

use std::collections::BTreeMap;

pub(crate) struct ParameterDatabase {
    /// We use a `BTreeMap` so we can get, say, all the versions of a particular `parameter_number`
    /// using `BTreeMap.range`.
    numeric_id_to_param: BTreeMap<NumericId, Parameter>,

    // TODO: Empirically test if we actually need the value to be a `BTreeSet` (instead of just a
    // `NumericId`). In other words, check if any GRIB abbreviations map to multiple parameters.
    abbrev_to_numeric_id: HashMap<Abbreviation, BTreeSet<NumericId>>,
}

impl ParameterDatabase {
    pub(crate) fn new() -> Self {
        Self {
            numeric_id_to_param: BTreeMap::new(),
            abbrev_to_numeric_id: HashMap::new(),
        }
    }

    pub(crate) fn insert(
        &mut self,
        numeric_id: NumericId,
        parameter: Parameter,
    ) -> Result<(), ParameterInsertionError> {
        // Insert into or modify `abbrev_to_numeric_id`:
        let mut numeric_id_is_unique = true;
        self.abbrev_to_numeric_id
            .entry(parameter.abbreviation.clone())
            .and_modify(|set| {
                numeric_id_is_unique = set.insert(numeric_id);
            })
            .or_insert(BTreeSet::from([numeric_id]));
        if !numeric_id_is_unique {
            return Err(
                ParameterInsertionError::NumericIdAlreadyExistsInAbbrevToNumericId((
                    numeric_id, parameter,
                )),
            );
        };

        // Insert into `numeric_id_to_param`:
        match self.numeric_id_to_param.insert(numeric_id, parameter) {
            None => Ok(()),
            Some(old_param) => Err(
                ParameterInsertionError::NumericIdAlreadyExistsInNumericIdToParam((
                    numeric_id, old_param,
                )),
            ),
        }
    }

    pub(crate) fn abbreviation_to_parameter(
        &self,
        abbreviation: &Abbreviation,
    ) -> AbbrevToParameter {
        let numeric_ids = match self.abbrev_to_numeric_id.get(abbreviation) {
            None => return AbbrevToParameter::AbbrevNotFound,
            Some(numeric_ids) => numeric_ids,
        };
        if numeric_ids.len() == 1 {
            let numeric_id = numeric_ids.first().unwrap();
            let param = self.numeric_id_to_param.get(&numeric_id).unwrap();
            AbbrevToParameter::Unique((numeric_id, param))
        } else {
            AbbrevToParameter::Multiple(
                numeric_ids
                    .iter()
                    .map(|numeric_id| {
                        let param = self.numeric_id_to_param.get(numeric_id).unwrap();
                        (numeric_id, param)
                    })
                    .collect(),
            )
        }
    }

    pub(crate) fn numeric_id_to_parameter(&self, numeric_id: &NumericId) -> Option<&Parameter> {
        self.numeric_id_to_param.get(numeric_id)
    }

    pub(crate) fn len(&self) -> usize {
        self.numeric_id_to_param.len()
    }
}

#[derive(thiserror::Error, Debug, derive_more::Display)]
#[display("{:?}, {:?}", _0.0, _0.1)]
pub(crate) enum ParameterInsertionError {
    NumericIdAlreadyExistsInAbbrevToNumericId((NumericId, Parameter)),
    NumericIdAlreadyExistsInNumericIdToParam((NumericId, Parameter)),
}

// TODO: Consider if perhaps we should replace `AbbrevtoParameter` with
//       `Option<Vec(&'a NumericId, &'a Parameter)>` or just `Vec`.
pub(crate) enum AbbrevToParameter<'a> {
    Unique((&'a NumericId, &'a Parameter)),
    Multiple(Vec<(&'a NumericId, &'a Parameter)>),
    AbbrevNotFound,
}

#[cfg(test)]
mod test {

    use crate::parameter::Status;

    use super::*;

    #[test]
    fn insert_and_retrieve() -> anyhow::Result<()> {
        let numeric_id = NumericId::new(0, 0, 0, 0, 0, 0);

        let abbreviation = Abbreviation("FOO".to_string());

        let param = Parameter {
            description: "Foo".to_string(),
            note: "Bar".to_string(),
            unit: "K".to_string(),
            abbreviation: abbreviation.clone(),
            status: Status::Operational,
        };

        let mut param_db = ParameterDatabase::new();
        assert_eq!(param_db.len(), 0);

        param_db.insert(numeric_id.clone(), param.clone())?;
        assert_eq!(param_db.len(), 1);

        let retrieved_param = param_db.abbreviation_to_parameter(&param.abbreviation);

        if let AbbrevToParameter::Unique((_, unique_param)) = retrieved_param {
            assert_eq!(param, *unique_param);
        } else {
            panic!("Failed to map from abbrev to param");
        }

        Ok(())
    }
}
