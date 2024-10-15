use crate::center_and_table_versions::CenterAndTableVersions;

pub(crate) mod hydrological;
pub(crate) mod meteorological;

pub(crate) trait Category {
    fn from_category_and_parameter_numbers(
        category_num: u8,
        parameter_num: u8,
        center_and_table_versions: &CenterAndTableVersions,
    ) -> Option<Self>
    where
        Self: Sized;
}
