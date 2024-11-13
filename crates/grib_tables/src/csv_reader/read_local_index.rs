use super::csv_path;

use std::collections::HashMap;
use std::num::IntErrorKind;

use anyhow::Context;
use serde::Deserialize;

use serde::Deserializer;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct GdalLocalIndex {
    pub(crate) center_code: u16,

    #[serde(deserialize_with = "deserialize_subcenter_code")]
    pub(crate) subcenter_code: u8,

    pub(crate) filename: String,
}

pub(crate) fn deserialize_subcenter_code<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer).unwrap();
    let result: Result<u16, _> = s.parse();
    match result {
        Ok(value) => match value {
            u16::MAX => Ok(u8::MAX),
            v => v
                .try_into()
                .with_context(|| format!("Failed to convert {value:?} into a u8 subcenter code!"))
                .map_err(serde::de::Error::custom),
        },
        Err(err) if *err.kind() == IntErrorKind::Empty => Ok(u8::MAX),
        Err(err) => Err(serde::de::Error::custom(err)),
    }
}

/// The values of the HashMap are the (center_code, subcenter_code).
pub(crate) fn get_local_index() -> HashMap<String, (u16, u8)> {
    let path = csv_path().join("grib2_table_4_2_local_index.csv");
    let mut reader = csv::Reader::from_path(&path)
        .with_context(|| format!("Failed: csv::Reader::from_path({path:?})"))
        .unwrap();
    let mut map = HashMap::new();
    for row in reader.deserialize() {
        let record: GdalLocalIndex = row
            .with_context(|| format!("Failed to deserialize row from {path:?}"))
            .unwrap();

        // Skip duplicate:
        if map.contains_key(&record.filename)
            && record.filename == "grib2_table_4_2_local_NDFD.csv"
            && record.center_code == 8
            && record.subcenter_code == u8::MAX
        {
            continue;
        }

        let center_and_subcenter_codes = (record.center_code, record.subcenter_code);
        match map.insert(record.filename, center_and_subcenter_codes) {
            None => (),
            Some(old_value) => panic!(
                "{path:?} contains duplicate filenames! Old center_code={}, old subcenter_code={}",
                old_value.0, old_value.1
            ),
        }
    }
    map
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_local_index() {
        let local_index = get_local_index();
        assert_eq!(local_index.len(), 5);
    }
}
