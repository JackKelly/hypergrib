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
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
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
            _ => Err(serde::de::Error::custom(format!(
                "Failed to parse parameter: {s}"
            ))),
        }
    }
}
