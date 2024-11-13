use core::fmt;

const N_BITS_PER_BYTE: u64 = 8;

pub struct NumericIdBuilder {
    product_discipline: u8,
    parameter_category: u8,
    parameter_number: u8,
    master_table_version: u8,
    originating_center: u16,
    local_table_version: u8,
}

impl NumericIdBuilder {
    pub(crate) fn new(
        product_discipline: u8,
        parameter_category: u8,
        parameter_number: u8,
    ) -> Self {
        Self {
            product_discipline,
            parameter_category,
            parameter_number,
            master_table_version: u8::MAX,
            originating_center: u16::MAX,
            local_table_version: u8::MAX,
        }
    }

    pub(crate) fn set_master_table_version(&mut self, master_table_version: u8) -> &Self {
        self.master_table_version = master_table_version;
        self
    }

    pub(crate) fn set_originating_center(&mut self, originating_center: u16) -> &Self {
        self.originating_center = originating_center;
        self
    }

    pub(crate) fn set_local_table_version(&mut self, local_table_version: u8) -> &Self {
        self.local_table_version = local_table_version;
        self
    }

    pub(crate) fn build(self) -> NumericId {
        NumericId::new(
            self.product_discipline,
            self.parameter_category,
            self.parameter_number,
            self.master_table_version,
            self.originating_center,
            self.local_table_version,
        )
    }
}

/// `NumericId` stores the unique numerical identifier for each GRIB `Parameter` as a single `u64`.
///
/// The components of the numerical ID are positioned into a single `u64` as follows:
/// (The right-most byte is byte 0):
///
/// | Byte  | Description          | dtype |     CSV file       |
/// |-------|----------------------|-------|--------------------|
/// | 7     | Zero (not used)      |  u8   |                    |
/// | 6     | product_discipline   |  u8   |                    |
/// | 5     | parameter_category   |  u8   |                    |
/// | 4     | parameter_number     |  u8   |                    |
/// | 3     | master_table_version |  u8   |                    |
/// | 1 & 2 | originating_center   |  u16  | wmo-im/CCT/c11.csv |
/// | 0     | local_table_version  |  u8   |                    |
///
/// In this way, we can, for example, get all parameters for a given category by
/// getting a `range` from the `BTreeMap`
/// from `0x00_<product_discipline>_<parameter_category>_00_00_00_00_00`
/// to   `0x00_<product_discipline>_<parameter_category>_FF_FF_FF_FF_FF`
///
/// `NumericId` is a `u64` because `NumericId` is used as the key in a `BTreeMap`, and `u64`s
/// are very fast to compare. (And `BTreeMaps` frequently compare keys!)
#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone)]
pub struct NumericId(u64);

impl NumericId {
    const PRODUCT_DISCIPLINE_BYTE: u64 = 6;
    const PARAMETER_CATEGORY_BYTE: u64 = 5;
    const PARAMETER_NUMBER_BYTE: u64 = 4;
    const MASTER_TABLE_VERSION_BYTE: u64 = 3;
    const ORIGINATING_CENTER_LEFT_BYTE: u64 = 2;
    const ORIGINATING_CENTER_RIGHT_BYTE: u64 = 1;
    const LOCAL_TABLE_VERSION_BYTE: u64 = 0;

    /// Create a new `NumericId`.
    ///
    /// `originating_center` and `local_table_version` must be `u16::MAX` and `u8::MAX`
    /// respectively for parameters which belong to the master table. This is consistent with
    /// the GRIB spec, which uses `u16::MAX` and `u8::MAX` to indicate a missing value.
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

impl fmt::Debug for NumericId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NumericId(discipline={}, category={}, parameter_number={}, \
                master_table_version={}, originating_center={}, local_table_version={}, u64 encoding={})",
            self.product_discipline(),
            self.parameter_category(),
            self.parameter_number(),
            self.master_table_version(),
            self.originating_center(),
            self.local_table_version(),
            self.0,
        )
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
}
