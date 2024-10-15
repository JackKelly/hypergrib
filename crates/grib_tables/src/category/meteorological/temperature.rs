use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{
    center_and_table_versions::CenterAndTableVersions, originating_center::OriginatingCenter,
    Parameter,
};

#[derive(FromPrimitive)]
pub(crate) enum TemperatureParameter {
    Temperature = 0,
    VirtualTemperature,
    PotentialTemperature,
    PseudoAdiabaticPotentialTemperature,
    MaximumTemperature,
    MinimumTemperature,
    DewPointTemperature,
    DewPointDepression,
    LapseRate,
    // etc.

    // NCEP local:
    NcepSnowPhaseChangeHeatFlux,
    NcepTemperatureTendencyByAllRadiation,
    // etc.
}

impl Parameter for TemperatureParameter {
    fn from_parameter_num(
        parameter_num: u8,
        center_and_table_versions: CenterAndTableVersions,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        if parameter_num < 192 {
            TemperatureParameter::from_u8(parameter_num)
        } else {
            // Parameter numbers >= 194 are reserved for local use:
            match center_and_table_versions.originating_center() {
                OriginatingCenter::NCEP {
                    local_table_version: _,
                } => match parameter_num {
                    192 => Some(TemperatureParameter::NcepSnowPhaseChangeHeatFlux),
                    193 => Some(TemperatureParameter::NcepTemperatureTendencyByAllRadiation),
                    _ => todo!(),
                },
            }
        }
    }

    fn abbrev(&self) -> &'static str {
        // This gets compiled to a jump table, which is O(1). See:
        // https://www.reddit.com/r/rust/comments/31kras/are_match_statements_constanttime_operations/
        match *self {
            TemperatureParameter::Temperature => "TMP",
            TemperatureParameter::VirtualTemperature => "VTMP",
            TemperatureParameter::PotentialTemperature => "POT",
            TemperatureParameter::PseudoAdiabaticPotentialTemperature => "EPOT",
            TemperatureParameter::MaximumTemperature => "TMAX",
            TemperatureParameter::MinimumTemperature => "TMIN",
            TemperatureParameter::DewPointTemperature => "DPT",
            TemperatureParameter::DewPointDepression => "DEPR",
            TemperatureParameter::LapseRate => "LAPR",
            // etc.

            // Local to NCEP:
            TemperatureParameter::NcepSnowPhaseChangeHeatFlux => "SNOHF",
            TemperatureParameter::NcepTemperatureTendencyByAllRadiation => "TTRAD",
        }
    }

    fn name(&self) -> &'static str {
        match *self {
            TemperatureParameter::Temperature => "Temperature",
            TemperatureParameter::VirtualTemperature => "Virtual temperature",
            _ => todo!(), // etc...
        }
    }

    fn unit(&self) -> &'static str {
        todo!();
    }
}
