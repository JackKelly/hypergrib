#[doc = include_str!("../README.md")]
use anyhow;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use serde::Deserialize;

#[derive(PartialEq, Debug, serde::Deserialize)]
struct IdxRecord {
    msg_id: u32,
    byte_offset: u32,
    #[serde(deserialize_with = "deserialize_init_datetime")]
    init_datetime: DateTime<Utc>,
    product: String, // TODO: Use Product enum?
    level: String,   // TODO: Use VerticalLevel enum?
    #[serde(deserialize_with = "deserialize_step")]
    step: TimeDelta,
    ens_member: Option<String>, // TODO: Use EnsembleMember enum?
}

// TODO: Return an iterator where each item is a `Result<IdxRecord>`.
fn parse_idx(b: &[u8]) -> anyhow::Result<Vec<IdxRecord>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b':')
        .has_headers(false)
        .from_reader(b);
    let mut records = vec![];
    for result in rdr.deserialize() {
        records.push(result?);
    }
    Ok(records)
}

pub fn deserialize_init_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut s = String::deserialize(deserializer)?;
    s.push_str("00"); // Hack because `parse_from_str` requires that the input string includes
                      // both hours and minutes, and GRIB `.idx` files don't contain minutes.
    NaiveDateTime::parse_from_str(&s, "d=%Y%m%d%H%M")
        .map(|ndt| ndt.and_utc())
        .map_err(|e| serde::de::Error::custom(format!("Invalid init_datetime: {e}")))
}

pub fn deserialize_step<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "anl" => Ok(TimeDelta::zero()),
        // TODO: Implement other strings!
        _ => Err(serde::de::Error::custom(format!(
            "Failed to parse forecast step: {s}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    #[test]
    fn test_parse_idx() -> anyhow::Result<()> {
        let idx_text = "\
1:0:d=2017010100:HGT:10 mb:anl:ENS=low-res ctl
2:50487:d=2017010100:TMP:10 mb:anl:ENS=low-res ctl
3:70653:d=2017010100:RH:10 mb:anl:ENS=low-res ctl
4:81565:d=2017010100:UGRD:10 mb:anl:ENS=low-res ctl
";
        let records = parse_idx(idx_text.as_bytes())?;
        assert_eq!(records.len(), 4);
        assert_eq!(
            records[0],
            IdxRecord {
                msg_id: 1,
                byte_offset: 0,
                init_datetime: NaiveDate::from_ymd_opt(2017, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc(),
                product: String::from("HGT"),
                level: String::from("10 mb"),
                step: TimeDelta::zero(),
                ens_member: Some(String::from("ENS=low-res ctl")),
            }
        );
        Ok(())
    }
}