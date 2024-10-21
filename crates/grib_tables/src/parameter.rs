use std::collections::{BTreeMap, BTreeSet, HashMap};

const N_BITS_PER_BYTE: u64 = 8;

/// `NumericId` is a `u64` because `NumericId` is used as the key in a `BTreeMap`, and `u64`s
/// are very fast to compare. (And `BTreeMaps` frequently compare keys!)
#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone, Debug, derive_more::Display)]
pub struct NumericId(u64);

impl NumericId {
    const PRODUCT_DISCIPLINE_BYTE: u64 = 6;
    const PARAMETER_CATEGORY_BYTE: u64 = 5;
    const PARAMETER_NUMBER_BYTE: u64 = 4;
    const MASTER_TABLE_VERSION_BYTE: u64 = 3;
    const ORIGINATING_CENTER_LEFT_BYTE: u64 = 2;
    const ORIGINATING_CENTER_RIGHT_BYTE: u64 = 1;
    const LOCAL_TABLE_VERSION_BYTE: u64 = 0;

    /// `originating_center` and `local_table_version` must be `u16::MAX` and `u8::MAX`
    /// respectively for parameters which belong to the master table.
    ///
    /// The input parameters are positioned into a single `u64` as follows:
    /// (The right-most byte is byte 0):
    ///
    /// - Byte 7: Zero (not used)
    /// - Byte 6: product_discipline (u8)
    /// - Byte 5: parameter_category (u8)
    /// - Byte 4: parameter_number (u8)
    /// - Byte 3: master_table_version (u8)
    /// - Bytes 1 & 2: originating_center (u16)
    /// - Byte 0: local_table_version (u8)
    ///
    /// In this way, we can get all parameters for a given category by getting a `range` from
    /// the `BTreeMap` from
    /// `0x00_<product_discipline>_<parameter_category>_00_00_00_00_00` to
    /// `0x00_<product_discipline>_<parameter_category>_FF_FF_FF_FF_FF`
    ///
    /// TODO: Passing in 6 ints is ugly and error-prone. Let's pass in a struct. Or use a builder
    /// pattern so the calling code can easily see which parameter is which!
    pub fn new(
        product_discipline: u8,
        parameter_category: u8,
        parameter_number: u8,
        master_table_version: u8,
        originating_center: u16,
        local_table_version: u8,
    ) -> Self {
        let numeric_id = shift_left_by_n_bytes(product_discipline, Self::PRODUCT_DISCIPLINE_BYTE)
            | shift_left_by_n_bytes(parameter_category, Self::PARAMETER_CATEGORY_BYTE)
            | shift_left_by_n_bytes(parameter_number, Self::PARAMETER_NUMBER_BYTE)
            | shift_left_by_n_bytes(master_table_version, Self::MASTER_TABLE_VERSION_BYTE)
            | shift_left_by_n_bytes(originating_center, Self::ORIGINATING_CENTER_RIGHT_BYTE)
            | (local_table_version as u64);
        Self(numeric_id)
    }

    pub fn product_discipline(&self) -> u8 {
        self.extract_nth_byte(Self::PRODUCT_DISCIPLINE_BYTE)
    }

    pub fn parameter_category(&self) -> u8 {
        self.extract_nth_byte(Self::PARAMETER_CATEGORY_BYTE)
    }

    pub fn parameter_number(&self) -> u8 {
        self.extract_nth_byte(Self::PARAMETER_NUMBER_BYTE)
    }

    pub fn master_table_version(&self) -> u8 {
        self.extract_nth_byte(Self::MASTER_TABLE_VERSION_BYTE)
    }

    pub fn originating_center(&self) -> u16 {
        let left = self.extract_nth_byte(Self::ORIGINATING_CENTER_LEFT_BYTE) as u16;
        let right = self.extract_nth_byte(Self::ORIGINATING_CENTER_RIGHT_BYTE) as u16;
        (left << N_BITS_PER_BYTE) | right
    }

    pub fn local_table_version(&self) -> u8 {
        self.extract_nth_byte(Self::LOCAL_TABLE_VERSION_BYTE)
    }

    /// This function counts the bytes from the right to the left.
    /// To extract the right-most byte, set `nth_byte` to 0. To extract the left-most byte, set
    /// `nth_byte` to 7.
    fn extract_nth_byte(&self, nth_byte: u64) -> u8 {
        debug_assert!(nth_byte < 8, "nth_byte must be < 8, not {}", nth_byte);
        let n_bits_to_shift = N_BITS_PER_BYTE * nth_byte;
        let bit_mask = (0xFF as u64) << n_bits_to_shift;
        let masked_and_shifted = (self.0 & bit_mask) >> n_bits_to_shift;
        debug_assert!(masked_and_shifted <= 0xFF);
        masked_and_shifted as u8
    }
}

fn shift_left_by_n_bytes<T>(value_to_shift: T, n_bytes: u64) -> u64
where
    u64: From<T>,
{
    debug_assert!(n_bytes < 8, "n_bytes must be < 8, not {}", n_bytes);
    let n_bits = N_BITS_PER_BYTE * n_bytes;
    u64::from(value_to_shift) << n_bits
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, derive_more::Display)]
struct Abbreviation(String);

#[derive(Clone, Debug, derive_more::Display, PartialEq, Eq)]
#[display("({}, {}, {}, {}, {})", description, note, unit, abbreviation, status)]
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
#[display("{:?}, {:?}", _0.0, _0.1)]
enum ParameterInsertionError {
    NumericKeyAlreadyExists((NumericId, Parameter)),
}

// TODO: Consider if perhaps we should replace `AbbrevtoParameter` with
// `Option<Vec(&'a NumericId, &'a Parameter)>`
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
            return Err(ParameterInsertionError::NumericKeyAlreadyExists((
                numeric_id, parameter,
            )));
        }
        let insert_option = self.numeric_id_to_params.insert(numeric_id, parameter);
        assert!(insert_option.is_none(), "insertion into numeric_id_to_params should return None here because we test for `contains_key()` above.");
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
    use super::*;

    #[test]
    fn test_extract_nth_byte_all_zero() {
        let numeric_id = NumericId(0x00_00_00_00_00_00_00_00);
        for i in 0..8 {
            assert_eq!(numeric_id.extract_nth_byte(i), 0);
        }
    }

    #[test]
    fn test_extract_nth_byte_all_ones() {
        let numeric_id = NumericId(0xFF_FF_FF_FF_FF_FF_FF_FF);
        for i in 0..8 {
            assert_eq!(numeric_id.extract_nth_byte(i), 0xFF);
        }
    }

    #[test]
    fn test_extract_nth_ff_byte() {
        for n in 0..8 {
            let numeric_id = NumericId((0xFF as u64) << (N_BITS_PER_BYTE * n));
            println!("{n} = {:#018x}", numeric_id.0);
            for i in 0..8 {
                if i == n {
                    assert_eq!(numeric_id.extract_nth_byte(n), 0xFF);
                } else {
                    assert_eq!(numeric_id.extract_nth_byte(i), 0);
                }
            }
        }
    }

    #[test]
    fn test_numeric_id() {
        let numeric_id = NumericId::new(0, 1, 2, 3, 400, 5);
        assert_eq!(numeric_id.product_discipline(), 0);
        assert_eq!(numeric_id.parameter_category(), 1);
        assert_eq!(numeric_id.parameter_number(), 2);
        assert_eq!(numeric_id.master_table_version(), 3);
        assert_eq!(numeric_id.originating_center(), 400);
        assert_eq!(numeric_id.local_table_version(), 5);
    }

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
