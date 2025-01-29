use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, Utc};
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
    parameter_sets: HashMap<String, HashMap<String, Vec<ParameterFilter>>>,
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
    start: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
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
struct ParameterFilter {
    include_vertical_levels: Option<Vec<String>>,
    exclude_vertical_levels: Option<Vec<String>>,
    include_forecast_steps: Option<Vec<u32>>,
    exclude_forecast_steps: Option<Vec<u32>>,
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
    use chrono::{TimeZone, Utc};
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
        assert_eq!(nwp.datasets.len(), 1);
        let dataset = &nwp.datasets[0];
        assert_eq!(dataset.dataset_id, "v12_atmos_0.5_degree");
        assert_eq!(dataset.data_files.file_type, FileType::Grib2);
        assert_eq!(dataset.data_files.extension, ".grib");
        assert_eq!(
            dataset.data_files.bucket_url,
            Url::parse("s3://noaa-gefs-pds/").unwrap()
        );
        assert_eq!(
            dataset.reference_datetimes.start,
            Utc.with_ymd_and_hms(2020, 9, 23, 12, 0, 0).unwrap()
        );
        assert_eq!(dataset.reference_datetimes.end, None);
        assert_eq!(dataset.reference_datetimes.number_of_daily_cycles, 4);
        assert_eq!(dataset.forecast_steps.len(), 3);
        let param_set_a = dataset
            .parameter_sets
            .get("a")
            .expect("Failed to find parameter_set 'a'");
        assert_eq!(
            param_set_a["DSWRF"][0].include_vertical_levels,
            Some(vec!["surface".to_string()])
        );
        assert!(param_set_a["TMP,RH"][0].include_forecast_steps.is_none());
    }
}
