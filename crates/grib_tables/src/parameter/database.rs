use std::collections::BTreeSet;

use std::collections::HashMap;

use super::{numeric_id::NumericId, Abbrev, Parameter};

use std::collections::BTreeMap;

pub(crate) struct ParameterDatabase {
    /// We use a `BTreeMap` so we can get, say, all the versions of a particular `parameter_number`
    /// using `BTreeMap.range`.
    numeric_id_to_param: BTreeMap<NumericId, Parameter>,

    // TODO: Empirically test if we actually need the value to be a `BTreeSet` (instead of just a
    // `NumericId`). In other words, check if any GRIB abbreviations map to multiple parameters.
    abbrev_to_numeric_id: HashMap<Abbrev, BTreeSet<NumericId>>,
}

impl ParameterDatabase {
    pub(crate) fn new() -> Self {
        Self {
            numeric_id_to_param: BTreeMap::new(),
            abbrev_to_numeric_id: HashMap::new(),
        }
    }

    /// Silently skips insertion into `abbrev_to_numeric_id` if abbrev = "".
    pub(crate) fn insert(
        &mut self,
        numeric_id: NumericId,
        parameter: Parameter,
    ) -> Result<(), ParameterInsertionError> {
        // Update abbrev_to_numeric_id:
        let numeric_id_is_unique = self.update_abbrev_to_numeric_id(numeric_id, &parameter);
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

    /// Returns true if `numeric_id` is unique within the set of `numeric_id`s associated with
    /// `parameter.abbrev`.
    ///
    /// If `parameter.abbrev` == "" then silently skips insertion into `abbrev_to_numeric_id` and returns true.
    fn update_abbrev_to_numeric_id(
        &mut self,
        numeric_id: NumericId,
        parameter: &Parameter,
    ) -> bool {
        let mut numeric_id_is_unique = true;
        if parameter.abbrev.0 != "" {
            self.abbrev_to_numeric_id
                .entry(parameter.abbrev.clone())
                .and_modify(|set| {
                    numeric_id_is_unique = set.insert(numeric_id);
                })
                .or_insert(BTreeSet::from([numeric_id]));
        }
        numeric_id_is_unique
    }

    pub(crate) fn abbrev_to_parameter(&self, abbrev: &Abbrev) -> Vec<(&NumericId, &Parameter)> {
        match self.abbrev_to_numeric_id.get(abbrev) {
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
#[display("ParameterInsertionError! {_variant}")]
pub(crate) enum ParameterInsertionError {
    #[display("NumericIdAlreadyExistsInAbbrevToNumericId\n  numeric_id={:?},\n  parameter={:?}", _0.0, _0.1)]
    NumericIdAlreadyExistsInAbbrevToNumericId((NumericId, Parameter)),
    #[display("NumericIdAlreadyExistsInNumericIdToParam\n  numeric_id={:?},\n  previously existing parameter={:?}", _0.0, _0.1)]
    NumericIdAlreadyExistsInNumericIdToParam((NumericId, Parameter)),
}

#[cfg(test)]
mod test {

    use crate::parameter::numeric_id::NumericIdBuilder;

    use super::*;

    #[test]
    fn insert_and_retrieve() -> anyhow::Result<()> {
        let numeric_id = NumericIdBuilder::new(0, 0, 0).build();

        let param = Parameter {
            abbrev: Abbrev("FOO".to_string()),
            name: "Foo".to_string(),
            unit: "K".to_string(),
        };

        let mut param_db = ParameterDatabase::new();
        assert_eq!(param_db.len(), 0);

        param_db.insert(numeric_id.clone(), param.clone())?;
        assert_eq!(param_db.len(), 1);

        let retrieved_params = param_db.abbrev_to_parameter(&param.abbrev);
        assert_eq!(retrieved_params.len(), 1);
        let (retrieved_numeric_id, unique_param) = retrieved_params.first().unwrap();
        assert_eq!(&numeric_id, *retrieved_numeric_id);
        assert_eq!(&param, *unique_param);

        Ok(())
    }
}
