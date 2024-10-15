use moisture::MoistureParameter;
use temperature::TemperatureParameter;

use crate::{center_and_table_versions::CenterAndTableVersions, Parameter};

use super::Category;

pub(crate) mod moisture;
pub(crate) mod temperature;

pub(crate) enum MeteorologicalCategory {
    Temperature(TemperatureParameter),
    Moisture(MoistureParameter),
    // etc.
}

impl Category for MeteorologicalCategory {
    fn from_category_and_parameter_numbers(
        category_num: u8,
        parameter_num: u8,
        center_and_table_versions: &CenterAndTableVersions,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        match category_num {
            0 => Some(MeteorologicalCategory::Temperature(
                TemperatureParameter::from_parameter_num(parameter_num, center_and_table_versions)?,
            )),
            1 => Some(MeteorologicalCategory::Moisture(
                MoistureParameter::from_parameter_num(parameter_num, center_and_table_versions)?,
            )),
            _ => None,
        }
    }
}
