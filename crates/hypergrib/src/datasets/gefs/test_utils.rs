use chrono::NaiveDateTime;
use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;

use super::Version;

#[derive(PartialEq, Debug, serde::Deserialize, Clone)]
pub(super) struct GfsTest {
    pub(super) path: String,
    #[serde(deserialize_with = "deserialize_gefs_version_enum")]
    pub(super) gefs_version_enum_variant: Version,
    #[serde(deserialize_with = "deserialize_reference_datetime")]
    pub(super) reference_datetime: DateTime<Utc>,
    pub(super) ensemble_member: String,
    #[serde(deserialize_with = "deserialize_forecast_hour")]
    pub(super) forecast_hour: TimeDelta,
}

fn deserialize_reference_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    let s = format!("{s}00");
    match NaiveDateTime::parse_from_str(&s, "%Y%m%dT%H%M") {
        Ok(dt) => Ok(dt.and_utc()),
        Err(e) => Err(serde::de::Error::custom(format!(
            "Invalid init datetime: {e}"
        ))),
    }
}

fn deserialize_gefs_version_enum<'de, D>(deserializer: D) -> Result<Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let variant_i = <usize>::deserialize(deserializer)?;
    Ok(Version::all_versions()[variant_i].clone())
}

fn deserialize_forecast_hour<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let forecast_hour = <i64>::deserialize(deserializer)?;
    Ok(TimeDelta::hours(forecast_hour))
}

pub(super) fn load_gefs_test_paths_csv() -> Vec<GfsTest> {
    // Gets the MANIFEST_DIR of the sub-crate.
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("src/datasets/gefs/test_paths.csv");
    let mut rdr = csv::Reader::from_path(&d).expect(format!("Failed to open {:?}", &d).as_str());
    let mut records = vec![];
    for result in rdr.deserialize() {
        records.push(result.unwrap());
    }
    records
}
