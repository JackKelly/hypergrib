#[doc = include_str!("../README.md")]
use anyhow;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeDelta, Utc};
use parameter::Parameter;
use serde::Deserialize;

mod parameter;

#[derive(PartialEq, Debug, serde::Deserialize)]
struct IdxRecord {
    msg_id: u32,
    byte_offset: u32,
    #[serde(deserialize_with = "deserialize_init_datetime")]
    init_datetime: DateTime<Utc>,
    parameter: Parameter,
    level: String,
    // TODO: Define `struct Level{
    //     fixed_surface_type: gribberish::templates::product::tables::FixedSurfaceType,
    //     value: Option<f32>
    // }`
    // e.g. "10 mb" would be `Level{FixedSurfaceType::IsobaricSurface, 10}`
    #[serde(deserialize_with = "deserialize_step")]
    step: TimeDelta,
    ens_member: Option<String>, // TODO: Define `enum EnsembleMember{Control, Perturbed(u16)}`?
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
    let s = <&str>::deserialize(deserializer)?;
    // The slightly convoluted approach below is necessary because `NaiveDateTime::parse_str`
    // requires the input string to include the hour but `.idx` files don't include hours!
    // So we _could_ implement a hack whereby we append "00" to the end of `s` but that requires
    // a heap allocation for every row of the `.idx`. The advantage of the approach below
    // is that it doesn't require any heap allocations.
    let (date, remainder) = NaiveDate::parse_and_remainder(s, "d=%Y%m%d")
        .map_err(|e| serde::de::Error::custom(format!("Invalid init date: {e}")))?;
    let hour: u32 = remainder.parse().map_err(|e| {
        serde::de::Error::custom(format!(
            "Hour of the NWP init could not be parsed into a u32: {e}"
        ))
    })?;
    match date.and_hms_opt(hour, 0, 0) {
        Some(dt) => Ok(dt.and_utc()),
        None => Err(serde::de::Error::custom(format!(
            "Invalid init hour: {hour}"
        ))),
    }
}

pub fn deserialize_step<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    match s {
        "anl" => Ok(TimeDelta::zero()),
        // TODO: Implement deser for other step strings! See:
        // https://github.com/NOAA-EMC/NCEPLIBS-grib_util/blob/develop/src/wgrib/wgrib.c#L2248-L2446
        // Even better, use existing strings from gribberish.
        _ => Err(serde::de::Error::custom(format!(
            "Failed to parse forecast step: {s}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use gribberish::templates::product::parameters::meteorological;

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
                parameter: Parameter::Mass(meteorological::MassProduct::GeopotentialHeight),
                level: String::from("10 mb"),
                step: TimeDelta::zero(),
                ens_member: Some(String::from("ENS=low-res ctl")),
            }
        );
        Ok(())
    }
}
