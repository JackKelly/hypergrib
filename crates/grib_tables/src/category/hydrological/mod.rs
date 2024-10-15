use crate::center_and_table_versions::CenterAndTableVersions;

use super::Category;

pub(crate) enum HydrologicalCategory {
    HydrologyBasicProduct, // TODO: Add embedded enum
}

impl Category for HydrologicalCategory {
    fn from_category_and_parameter_numbers(
        category_num: u8,
        parameter_num: u8,
        center_and_table_versions: &CenterAndTableVersions,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        todo!();
    }
}
