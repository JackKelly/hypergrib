//! NOAA's Global Ensemble Forecast System (GEFS).
//! https://registry.opendata.aws/noaa-gefs

use anyhow;
use chrono::prelude::*;
use chrono::TimeDelta;

#[derive(PartialEq, Debug, serde::Deserialize)]
struct IdxRecord {
    msg_id: u32,
    byte_offset: u32,
    init_time: String,      // TODO: Use DateTime<Utc>
    nwp_variable: String,   // TODO: Use NWPVariable enum?
    vertical_level: String, // TODO: Use VerticalLevel enum?
    forecast_step: String,  // TODO: Use TimeDelta?
    ensemble_member: String, // TODO: Use EnsembleMember enum?
                            // TODO: Add GRIB filename!
}

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

struct GefsKey {
    init_time: DateTime<Utc>,
    ensemble_member: u16, // Or an `enum EnsembleMember {Control, Perturbed(u16)}`
    forecast_step: TimeDelta,
    nwp_variable: String,   // TODO: Use NWPVariable enum?
    vertical_level: String, // TODO: Use VerticalLevel enum?
}

impl crate::Key for GefsKey {
    fn to_path(&self) -> object_store::path::Path {
        let mut parts = Vec::<object_store::path::PathPart>::with_capacity(3);
        let init_hour = format!("{:02}", self.init_time.hour());

        // First part of the Path:
        parts.push(self.init_time.format("gefs.%Y%m%d").to_string().into());

        // Second part of the Path:
        parts.push(init_hour.clone().into());

        // Third part of the Path:
        let ensemble_member = if self.ensemble_member == 0 {
            "gec00".to_string()
        } else {
            format!("gef{:02}", self.ensemble_member)
        };
        let forecast_step = if self.forecast_step == TimeDelta::zero() {
            "anl".to_string()
        } else {
            format!("f{:03}", self.forecast_step.num_hours())
        };
        parts.push(
            format!(
                "{ensemble_member}.t{init_hour}z.pgrb2a{forecast_step}",
                ensemble_member = ensemble_member,
                init_hour = init_hour,
                forecast_step = forecast_step,
            )
            .into(),
        );
        object_store::path::Path::from_iter(parts)
    }
}

impl TryFrom<object_store::path::Path> for GefsKey {
    type Error = &'static str;

    fn try_from(value: object_store::path::Path) -> Result<Self, Self::Error> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use crate::Key;

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
                init_time: String::from("d=2017010100"),
                nwp_variable: String::from("HGT"),
                vertical_level: String::from("10 mb"),
                forecast_step: String::from("anl"),
                ensemble_member: String::from("ENS=low-res ctl"),
            }
        );
        Ok(())
    }

    #[test]
    fn test_key_to_path() -> anyhow::Result<()> {
        let k = GefsKey {
            // Note that the string on the line below includes minutes, even though the GEFS
            // idx files do not contain minutes. This is because `chrono::NaiveDateTime::parse_from_str`
            // throws an error if minutes aren't present in the string :(.
            init_time: NaiveDateTime::parse_from_str("201701010000", "%Y%m%d%H%M")
                .expect("parse datetime")
                .and_utc(),
            ensemble_member: 0,
            forecast_step: TimeDelta::hours(6),
            nwp_variable: "HGT".to_string(),
            vertical_level: "10 mb".to_string(),
        };
        let p = k.to_path();
        assert_eq!(
            p,
            object_store::path::Path::from("gefs.20170101/00/gec00.t00z.pgrb2af006")
        );
        Ok(())
    }
}
