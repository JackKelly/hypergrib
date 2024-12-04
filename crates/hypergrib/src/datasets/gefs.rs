//! NOAA's Global Ensemble Forecast System (GEFS).
//! https://registry.opendata.aws/noaa-gefs

mod test_utils;
mod version;
pub(crate) use version::Version;

use chrono::{TimeDelta, Timelike};

struct Gefs;

impl crate::ToIdxPath for Gefs {
    fn to_idx_path(
        reference_datetime: &chrono::DateTime<chrono::Utc>,
        _parameter: &str,
        _vertical_level: &str,
        forecast_step: &TimeDelta,
        ensemble_member: Option<&str>,
    ) -> object_store::path::Path {
        // TODO: The code below only works for "old" (gefs::Version::V1) GEFS paths.
        // Change this function to work with all gefs::Versions. And, for "Version::V3",
        // have a `phf::Map` (or maybe just a `HashMap`) which tells us whether
        // the  parameter belongs to 'atmos', 'chem', 'wave'; and 'pgrb2a' or 'pgrb2b' etc.
        let mut parts = Vec::<object_store::path::PathPart>::with_capacity(3);

        // First part of the Path:
        parts.push(reference_datetime.format("gefs.%Y%m%d").to_string().into());

        // Second part of the Path:
        let init_hour = format!("{:02}", reference_datetime.hour());
        parts.push(init_hour.as_str().into());

        // Third part of the Path:
        let ensemble_member = ensemble_member.expect("GEFS requires the ensemble member!");
        let forecast_step = if *forecast_step == TimeDelta::zero() {
            "anl".to_string()
        } else {
            format!("f{:03}", forecast_step.num_hours())
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

#[cfg(test)]
mod tests {

    use crate::{ymdh_to_datetime, ToIdxPath};

    use super::*;

    #[test]
    fn test_to_idx_path() -> anyhow::Result<()> {
        // TODO: Once `Gefs::to_idx_path` knows how to output different paths for different
        // `GefsVersion`s, then update this test to use `GEFS_TEST_DATA`.
        let p = Gefs::to_idx_path(
            &ymdh_to_datetime(2017, 1, 1, 0),
            "HGT",
            "10 mb",
            &TimeDelta::hours(6),
            Some("gec00"),
        );
        assert_eq!(
            p,
            object_store::path::Path::from("gefs.20170101/00/gec00.t00z.pgrb2af006")
        );
        Ok(())
    }
}
