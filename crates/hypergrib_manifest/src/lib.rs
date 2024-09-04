use anyhow;

#[derive(PartialEq, Debug, serde::Deserialize)]
struct IdxRecord {
    msg_id: u32,
    byte_offset: u32,
    init_time: String, // TODO: Convert to datetime
    nwp_variable: String,
    vertical_level: String,
    step: String,
    member: String,
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

fn read_idx_into_duck_db(filename: String) -> anyhow::Result<()> {
    let conn = duckdb::Connection::open_in_memory()?;

    conn.execute_batch(
        r"CREATE TABLE grib_message (
            msg_id INTEGER,
            byte_offset INTEGER,
            init_time TIMESTAMP,
            nwp_variable VARCHAR,
            vertical_level VARCHAR,
            step_raw VARCHAR,
            step INTERVAL,
            ens_member VARCHAR,
            );
        ",
    )?;
    // TODO: Change to enums: nwp_variable, vertical_level, ensemble_member
    // TODO: Add filename of GRIB file (with full path)

    conn.execute_batch(
        r"COPY grib_message (msg_id, byte_offset, init_time, nwp_variable, vertical_level, step_raw, ens_member) FROM 
            '/home/jack/dev/rust/hypergrib/gec00.t00z.pgrb2af000.idx'
            (
              DELIMITER ':',
              FORMAT CSV,
              HEADER false,
              AUTO_DETECT false,
              TIMESTAMPFORMAT 'd=%Y%m%d%H'
            );
          ",
    )?;
    // TODO: Pass in filename argument!
    // TODO: Convert step_raw to step, maybe by first storing step_raw as VARCHAR, replacing 'anl'
    // with '0', taking just the first two characters of the string, converting that to an int, and
    // passing that to [`to_hours(integer)`](https://duckdb.org/docs/sql/functions/interval#to_hoursinteger)

    // Print some test data:
    let mut stmt = conn.prepare("SELECT * FROM grib_message WHERE msg_id < 5")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        println!("{:?}", row.get::<_, u32>(1)?);
    }

    Ok(())
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
                step: String::from("anl"),
                member: String::from("ENS=low-res ctl"),
            }
        );
        Ok(())
    }

    #[test]
    fn test_read_idx_into_duck_db() -> anyhow::Result<()> {
        read_idx_into_duck_db(String::from("FILENAME NOT USED YET!"))?;
        Ok(())
    }
}
