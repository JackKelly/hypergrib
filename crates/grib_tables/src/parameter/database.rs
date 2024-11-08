use std::collections::BTreeSet;

use std::collections::HashMap;

use super::{numeric_id::NumericId, Parameter, ShortName};

use std::collections::BTreeMap;

pub(crate) struct ParameterDatabase {
    /// We use a `BTreeMap` so we can get, say, all the versions of a particular `parameter_number`
    /// using `BTreeMap.range`.
    numeric_id_to_param: BTreeMap<NumericId, Parameter>,

    // TODO: Empirically test if we actually need the value to be a `BTreeSet` (instead of just a
    // `NumericId`). In other words, check if any GRIB abbreviations map to multiple parameters.
    abbrev_to_numeric_id: HashMap<ShortName, BTreeSet<NumericId>>,
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
            .entry(parameter.short_name.clone())
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
        abbreviation: &ShortName,
    ) -> Vec<(&NumericId, &Parameter)> {
        match self.abbrev_to_numeric_id.get(abbreviation) {
            None => vec![],
            Some(numeric_ids) => numeric_ids
                .iter()
                .map(|numeric_id| {
                    let param = self.numeric_id_to_param.get(numeric_id).unwrap();
                    (numeric_id, param)
                })
                .collect(),
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn insert_and_retrieve() -> anyhow::Result<()> {
        let numeric_id = NumericId::new(0, 0, 0, 0, 0, 0);

        let param = Parameter {
            short_name: ShortName("FOO".to_string()),
            name: "Foo".to_string(),
            unit: "K".to_string(),
        };

        let mut param_db = ParameterDatabase::new();
        assert_eq!(param_db.len(), 0);

        param_db.insert(numeric_id.clone(), param.clone())?;
        assert_eq!(param_db.len(), 1);

        let retrieved_params = param_db.abbreviation_to_parameter(&param.short_name);
        assert_eq!(retrieved_params.len(), 1);
        let (retrieved_numeric_id, unique_param) = retrieved_params.first().unwrap();
        assert_eq!(&numeric_id, *retrieved_numeric_id);
        assert_eq!(&param, *unique_param);

        Ok(())
    }
}
