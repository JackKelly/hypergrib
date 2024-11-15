use std::collections::BTreeSet;
use std::fmt::Write;

use std::collections::HashMap;

use crate::{csv_reader::{read_local_index::get_local_index, read_table_4_2::{gdal_master_table_4_2_iterator, gdal_table_4_2_iterator, list_gdal_table_4_2_csv_files}}, MASTER_TABLE_VERSION};

use super::{numeric_id::NumericId, Abbrev, Parameter};

use std::collections::BTreeMap;
use anyhow::Context;

pub struct ParameterDatabase {
    /// We use a `BTreeMap` so we can get, say, all the versions of a particular `parameter_number`
    /// using `BTreeMap.range`.
    numeric_id_to_param: BTreeMap<NumericId, Parameter>,

    // TODO: Empirically test if we actually need the value to be a `BTreeSet` (instead of just a
    // `NumericId`). In other words, check if any GRIB abbreviations map to multiple parameters.
    abbrev_to_numeric_id: HashMap<Abbrev, BTreeSet<NumericId>>,
}

impl ParameterDatabase {
    pub fn new() -> Self {
        Self {
            numeric_id_to_param: BTreeMap::new(),
            abbrev_to_numeric_id: HashMap::new(),
        }
    }

    /// Example:
    /// ```
    /// use grib_tables::ParameterDatabase;
    /// # fn main() -> anyhow::Result<()> {
    /// let param_db = ParameterDatabase::new().populate()?;
    /// assert_eq!(param_db.num_numeric_ids(), 1669);
    /// assert_eq!(param_db.num_abbrevs(), 1168);
    /// # Ok(())
    /// # }
    /// ```
    pub fn populate(mut self) -> anyhow::Result<Self> {
        let local_index = get_local_index();
        let re_master_table =
            regex::Regex::new(r"^grib2_table_4_2_(?<discipline>\d{1,2})_(?<category>\d{1,3}).csv$")
                .unwrap();
        let re_local_table = regex::Regex::new(r"^grib2_table_4_2_local_[A-Z][A-Za-z]+.csv$").unwrap();
        for path in list_gdal_table_4_2_csv_files()? {
            let path = path?;
            let file_name = path
                .file_name()
                .with_context(|| format!("Failed to get file_name from path {path:?}"))?
                .to_str()
                .with_context(|| format!("Failed to convert file_stem to &str for path {path:?}"))?;
            if file_name == "grib2_table_4_2_local_index.csv" {
                continue;
            } else if let Some(captures) = re_master_table.captures(file_name) {
                let discipline = (&captures["discipline"]).parse().expect("parse discipline");
                let category = (&captures["category"]).parse().expect("parse category");
                for record in gdal_master_table_4_2_iterator(discipline, category)? {
                    let (mut numeric_id_builder, parameter) = record;
                    numeric_id_builder.set_master_table_version(MASTER_TABLE_VERSION);
                    let numeric_id = numeric_id_builder.build();
                    self.insert(numeric_id, parameter).with_context(|| 
                        format!("Error when inserting into parameter database. Master table 4.2 path={path:?}")
                    )?;
                }
            } else if re_local_table.is_match(file_name) {
                for record in gdal_table_4_2_iterator(&path)? {
                    let (mut numeric_id_builder, parameter) = record.into();
                    numeric_id_builder.set_master_table_version(MASTER_TABLE_VERSION);
                    let (originating_center, subcenter) = local_index[file_name];
                    numeric_id_builder.set_originating_center(originating_center);
                    numeric_id_builder.set_subcenter(subcenter);
                    let numeric_id = numeric_id_builder.build();
                    self.insert(numeric_id, parameter).with_context(||
                        format!("Error when inserting into parameter database. Local table 4.2 path={path:?}")
                    )?;
                }
            } else {
                return Err(anyhow::format_err!("Failed to interpret CSV path {path:?}!"));
            }
        }
        Ok(self)
    }

    /// Returns a `Vec` because some abbreviations are associated with multiple parameters.
    /// See https://github.com/JackKelly/hypergrib/issues/20
    pub fn abbrev_to_parameter(&self, abbrev: &Abbrev) -> Vec<(&NumericId, &Parameter)> {
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

    pub fn num_numeric_ids(&self) -> usize {
        self.numeric_id_to_param.len()
    }

    pub fn num_abbrevs(&self) -> usize {
        self.abbrev_to_numeric_id.len()
    }

    pub fn numeric_id_to_param(&self) -> &BTreeMap<NumericId, Parameter> {
        &self.numeric_id_to_param
    }

    pub fn abbrev_to_numeric_id(&self) -> &HashMap<Abbrev, BTreeSet<NumericId>> {
        &self.abbrev_to_numeric_id
    }

    /// Silently skips insertion into `abbrev_to_numeric_id` if abbrev = "".
    fn insert(
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

    fn abbrevs_with_multiple_numeric_ids(&self) -> Vec<(&Abbrev, &BTreeSet<NumericId>)> {
        let mut abbrevs_with_multiple_numeric_ids: Vec<_> = self
            .abbrev_to_numeric_id
            .iter()
            .filter(|(_, numeric_ids)| numeric_ids.len() > 1)
            .collect();
        abbrevs_with_multiple_numeric_ids
            .sort_by(|(abbrev1, _), (abbrev2, _)| abbrev1.cmp(&abbrev2));
        abbrevs_with_multiple_numeric_ids
    }

    pub fn describe_abbrevs_with_multiple_params(&self) -> String {
        let mut s = String::new();
        let mut count = 0;
        self.abbrevs_with_multiple_numeric_ids()
            .iter()
            .for_each(|(abbrev, set_of_numeric_ids)| {
                count += 1;
                writeln!(s, "- {abbrev}:").expect("writeln");
                set_of_numeric_ids.into_iter().for_each(|numeric_id| {
                    let param = self.numeric_id_to_param.get(&numeric_id).unwrap();
                    writeln!(s, "    - name='{}', unit='{}',", param.name, param.unit)
                        .expect("writeln");
                    writeln!(
                        s,
                        "        - discipline={:2}, category={:3}, number={:3}, center={:5}, subcenter={:3}",
                        numeric_id.product_discipline(),
                        numeric_id.parameter_category(),
                        numeric_id.parameter_number(),
                        numeric_id.originating_center(),
                        numeric_id.subcenter(),
                    )
                    .expect("writeln");
                });
            });
        writeln!(
            s,
            "\n{} abbreviations are associated with multiple parameters.",
            count
        )
        .expect("writeln");
        s
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
    use crate::parameter::Abbrev;

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
        assert_eq!(param_db.num_numeric_ids(), 0);

        param_db.insert(numeric_id.clone(), param.clone())?;
        assert_eq!(param_db.num_numeric_ids(), 1);

        let retrieved_params = param_db.abbrev_to_parameter(&param.abbrev);
        assert_eq!(retrieved_params.len(), 1);
        let (retrieved_numeric_id, unique_param) = retrieved_params.first().unwrap();
        assert_eq!(&numeric_id, *retrieved_numeric_id);
        assert_eq!(&param, *unique_param);

        Ok(())
    }

    #[test]
    fn test_for_duplicate_abbreviations() -> anyhow::Result<()> {
        let  param_db = ParameterDatabase::new().populate()?;
        println!("{}", param_db.describe_abbrevs_with_multiple_params());
        Ok(())
    }
}
