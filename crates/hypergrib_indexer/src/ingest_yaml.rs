use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
struct NWP {
    name: String,
    description: String,
    #[serde(deserialize_with = "deserialize_urls")]
    documentation_urls: Vec<Url>,
    datasets: Vec<Dataset>,
}

#[derive(Debug, Deserialize)]
struct Dataset {
    dataset_id: String,
    nwp_model_version: f32,
    data_files: FileInfo,
    index_files: FileInfo,
    formatting_template: String,
    reference_datetimes: ReferenceDatetimes,
    ensemble_members: EnsembleMembers,
    analysis_step: String,
    forecast_steps: Vec<ForecastStepRange>,
    vertical_levels: Vec<String>,

    /// The key of the outer HashMap is the param set name (e.g. 'a' or 'b' in GEFS).
    /// The key of the inner HashMap is the NWP param abbreviation (e.g. 'TMP' or 'RH').
    parameter_sets: HashMap<String, HashMap<String, Vec<ParameterSetDetail>>>,
}

#[derive(Debug, Deserialize)]
struct FileInfo {
    file_type: FileType,
    extension: String,
    #[serde(deserialize_with = "deserialize_url")]
    bucket_url: Url,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum FileType {
    Grib2,
    Idx,
}

#[derive(Debug, Deserialize)]
struct ReferenceDatetimes {
    start: DateTime<FixedOffset>,
    end: Option<DateTime<FixedOffset>>,
    number_of_daily_cycles: u8,
}

#[derive(Debug, Deserialize)]
struct EnsembleMembers {
    control: Option<String>,
    perturbed: Option<PerturbedEnsembleMembers>,
    ens_mean: Option<String>,
    ens_spread: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PerturbedEnsembleMembers {
    formatting_template: String,
    start: u32,
    end: u32,
}

#[derive(Debug, Deserialize)]
struct ForecastStepRange {
    daily_cycles: Vec<u8>,
    start_hour: u32,
    end_hour: u32,
    step_duration_in_hours: u32,
}

#[derive(Debug, Deserialize)]
struct ParameterSetDetail {
    vertical_levels: Option<VerticalLevels>,
    forecast_steps: Option<ForecastSteps>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum VerticalLevels {
    IncludeOnly(IncludeOnly<String>),
    Exclude(Exclude<String>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ForecastSteps {
    IncludeOnly(IncludeOnly<u32>),
    Exclude(Exclude<u32>),
}

#[derive(Debug, Deserialize)]
struct IncludeOnly<T> {
    include_only: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct Exclude<T> {
    exclude: Vec<T>,
}

// Custom deserialization function for a single URL
fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Url::parse(&s).map_err(serde::de::Error::custom)
}

// Custom deserialization function for a vector of URLs
fn deserialize_urls<'de, D>(deserializer: D) -> Result<Vec<Url>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    strings
        .iter()
        .map(|s| Url::parse(s).map_err(serde::de::Error::custom))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn test_load_yaml() {
        // Construct the path relative to the Cargo.toml
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("datasets/gefs/index.yaml");

        // Open the file
        let mut file = File::open(&path).expect("Unable to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read file");

        // Deserialize the YAML contents into the NWP struct
        let nwp: NWP = serde_yaml::from_str(&contents).expect("Unable to parse YAML");

        // Add assertions here to verify the contents of the `nwp` struct
        // For example:
        assert_eq!(nwp.name, "GEFS");
        // Add more assertions as needed
    }
}
