#[doc = include_str!("../README.md")]
use anyhow;
use chrono::{DateTime, TimeDelta, Utc};

pub mod datasets;

#[derive(PartialEq, Debug, serde::Deserialize)]
struct IdxRecord {
    msg_id: u32,
    byte_offset: u32,
    init_datetime: String, // TODO: Use Datetime<Utc>
    product: String,       // TODO: Use Product enum?
    level: String,         // TODO: Use VerticalLevel enum?
    step: String,          // TODO: Use TimeDelta?
    ens_member: Option<String>, // TODO: Use EnsembleMember enum?
                           // TODO: msg_length: u32  // In bytes
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

trait ToIdxLocation {
    fn to_idx_location(
        init_datetime: DateTime<Utc>,
        product: String,
        level: String,
        step: TimeDelta,
        ens_member: Option<u32>,
    ) -> object_store::path::Path;
}

#[cfg(test)]
mod tests {
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
                init_datetime: String::from("d=2017010100"),
                product: String::from("HGT"),
                level: String::from("10 mb"),
                step: String::from("anl"),
                ens_member: Some(String::from("ENS=low-res ctl")),
            }
        );
        Ok(())
    }
}
