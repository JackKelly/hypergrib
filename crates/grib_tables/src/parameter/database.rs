use std::collections::BTreeSet;

use std::collections::HashMap;

use super::{numeric_id::NumericId, Abbreviation, Parameter};

use std::collections::BTreeMap;

pub(crate) struct ParameterDatabase {
    /// We use a `BTreeMap` so we can get, say, all the versions of a particular parameter_number
    /// using BTreeMap.range.
    pub(crate) numeric_id_to_params: BTreeMap<NumericId, Parameter>,

    /// TODO: Empirically test if we actually need the value to be a BTreeSet (instead of just a
    /// NumericId). In other words, check if any GRIB abbreviations map to multiple parameters.
    pub(crate) abbrev_to_numeric_id: HashMap<Abbreviation, BTreeSet<NumericId>>,
}

#[derive(thiserror::Error, Debug, derive_more::Display)]
#[display("{:?}, {:?}", _0.0, _0.1)]
pub(crate) enum ParameterInsertionError {
    NumericKeyAlreadyExists((NumericId, Parameter)),
}

// TODO: Consider if perhaps we should replace `AbbrevtoParameter` with
// `Option<Vec(&'a NumericId, &'a Parameter)>`
pub(crate) enum AbbrevToParameter<'a> {
    Unique((&'a NumericId, &'a Parameter)),
    Multiple(Vec<(&'a NumericId, &'a Parameter)>),
    AbbrevNotFound,
}

impl ParameterDatabase {
    pub(crate) fn new() -> Self {
        Self {
            numeric_id_to_params: BTreeMap::new(),
            abbrev_to_numeric_id: HashMap::new(),
        }
    }

    pub(crate) fn insert(
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
            return Err(ParameterInsertionError::NumericKeyAlreadyExists((
                numeric_id, parameter,
            )));
        }
        let insert_option = self.numeric_id_to_params.insert(numeric_id, parameter);
        assert!(insert_option.is_none(), "insertion into numeric_id_to_params should return None here because we test for `contains_key()` above.");
        Ok(())
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

    pub(crate) fn numeric_id_to_parameter(&self, numeric_id: &NumericId) -> Option<&Parameter> {
        self.numeric_id_to_params.get(numeric_id)
    }

    pub(crate) fn len(&self) -> usize {
        self.numeric_id_to_params.len()
    }
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
