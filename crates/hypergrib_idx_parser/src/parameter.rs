use std::str::FromStr;

use gribberish::templates::product::parameters::meteorological;

#[derive(PartialEq, Debug)]
pub(crate) enum Parameter {
    Temperature(meteorological::TemperatureProduct),
    Moisture(meteorological::MoistureProduct),
    Momentum(meteorological::MomentumProduct),
    Cloud(meteorological::CloudProduct),
    Mass(meteorological::MassProduct),
    Radar(meteorological::RadarProduct),
    ForecastRadarImagery(meteorological::ForecastRadarImagery),
    Electromagnetics(meteorological::Electromagnetics),
    PhysicalAtmosphericProperties(meteorological::PhysicalAtmosphericProperties),
}

impl<'de> serde::Deserialize<'de> for Parameter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Can we get rid of this String allocation?
        // Maybe using deserializer.deserialize_str(visitor)?
        // Or maybe using serde_byes::ByteBuf?
        let s = String::deserialize(deserializer)?;
        Parameter::from_str(&s).map_err(|e| serde::de::Error::custom(format!("{e:?}")))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseParameterError(String);

impl FromStr for Parameter {
    type Err = ParseParameterError;
    // TODO: Can we create a Rust macro to automatically map from `abbrev` strings that already
    // exist in `gribberish`? See https://github.com/mpiannucci/gribberish/issues/41#issuecomment-2386495107
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TMP" => Ok(Self::Temperature(
                meteorological::TemperatureProduct::Temperature,
            )),
            "HGT" => Ok(Self::Mass(meteorological::MassProduct::GeopotentialHeight)),
            "RH" => Ok(Self::Moisture(
                meteorological::MoistureProduct::RelativeHumidity,
            )),
            "UGRD" => Ok(Self::Momentum(
                meteorological::MomentumProduct::UComponentWindSpeed,
            )),
            // TODO: Implement deser for other parameter strings!
            _ => Err(ParseParameterError(format!(
                "Failed to parse parameter: {s}"
            ))),
        }
    }
}
