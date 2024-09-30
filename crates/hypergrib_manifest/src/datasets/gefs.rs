//! NOAA's Global Ensemble Forecast System (GEFS).
//! https://registry.opendata.aws/noaa-gefs

use crate::{Dataset, Manifest};
use anyhow;

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

struct GefsDataset {
    manifest: Manifest,
}

impl Dataset for GefsDataset {
    fn ingest_grib_idx(
        &mut self,
        idx_path: object_store::path::Path,
        idx_contents: &[u8],
    ) -> anyhow::Result<()> {
        // insert `idx_path` into `self.dataset.paths`, and get a ref to the `path` in `paths`
        // for use in the `Chunk`.
        todo!()
    }

    fn manifest_as_ref(&self) -> &Manifest {
        &self.manifest
    }
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
                init_time: String::from("d=2017010100"),
                nwp_variable: String::from("HGT"),
                vertical_level: String::from("10 mb"),
                forecast_step: String::from("anl"),
                ensemble_member: String::from("ENS=low-res ctl"),
            }
        );
        Ok(())
    }
}
