//! NOAA's Global Ensemble Forecast System (GEFS).
//! https://registry.opendata.aws/noaa-gefs

use std::{error::Error, fmt::Display, sync::Arc};

use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone, Timelike, Utc};
use futures_util::StreamExt;
use object_store::ObjectStore;
use serde::Deserialize;

use crate::filter_by_ext;

const BUCKET_ID: &str = "noaa-gefs-pds";
struct Gefs;

impl crate::ToIdxPath for Gefs {
    fn to_idx_path(
        reference_datetime: &chrono::DateTime<chrono::Utc>,
        _parameter: &str,
        _vertical_level: &str,
        forecast_step: &TimeDelta,
        ensemble_member: Option<&str>,
    ) -> object_store::path::Path {
        // TODO: The code below only works for "old" (GefsVersion::V1) GEFS paths. But GEFS switched to new,
        // more complicated paths at some point. For the hypergrib MVP, change this function to
        // only handle the new paths. Then implement both "old" and "new" and switch on the
        // `reference_datetime` to decide whether to use "old" or "new" format. And, for the "new"
        // format, have a `phf::Map` which tells us whether the parameter belongs to 'atmos',
        // 'chem', 'wave'; and 'pgrb2a' or 'pgrb2b' etc.
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

struct GefsCoordLabelsBuilder {
    coord_labels_builder: crate::CoordLabelsBuilder,
}

impl GefsCoordLabelsBuilder {
    fn new(
        grib_store: Arc<dyn ObjectStore>,
        grib_base_path: object_store::path::Path,
        idx_store: Arc<dyn ObjectStore>,
        idx_base_path: object_store::path::Path,
    ) -> Self {
        Self {
            coord_labels_builder: crate::CoordLabelsBuilder::new(
                grib_store,
                grib_base_path,
                idx_store,
                idx_base_path,
            ),
        }
    }

    async fn extract_from_idx_paths(&mut self) -> anyhow::Result<()> {
        // Get an `async` stream of Metadata objects:
        let list_stream = self
            .coord_labels_builder
            .idx_store
            .list(Some(&self.coord_labels_builder.idx_base_path));

        let mut list_stream = filter_by_ext(list_stream, "idx");

        // Loop through each .idx filename:
        while let Some(meta) = list_stream.next().await.transpose().unwrap() {
            let gefs_idx_path = GefsIdxPath::try_from(&meta.location)?;
            self.coord_labels_builder
                .reference_datetime
                .insert(gefs_idx_path.extract_reference_datetime()?);
        }
        Ok(())
    }
}

impl crate::GetCoordLabels for GefsCoordLabelsBuilder {
    async fn get_coord_labels(mut self) -> anyhow::Result<crate::CoordLabels> {
        self.extract_from_idx_paths().await?;
        // TODO: self.extract_from_gribs().await?; // Maybe do concurrently with
        // extract_from_idx_paths?
        Ok(self.coord_labels_builder.build())
    }
}

#[derive(Debug)]
struct GefsIdxError {
    path: String,
    error: String,
}

impl Display for GefsIdxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error: {}. For GEFS .idx path: {}",
            self.error, self.path
        )
    }
}

impl Error for GefsIdxError {}

/// The was the GEFS data is structured has changed over time.
/// We manually identified the boundaries from the following information sources:
/// - [NOAA GEFS page](https://www.emc.ncep.noaa.gov/emc/pages/numerical_forecast_systems/gefs.php)
///   which includes the model version numbers.
/// - [NCEP Products Inventory](https://www.nco.ncep.noaa.gov/pmb/products/gens/) which describes
///   how the filenames are formatted for the latest version of the model.
/// - Our main source of information was the [GEFS AWS S3 bucket](https://noaa-gefs-pds.s3.amazonaws.com/index.html).
///   Note that all the paths below are "real" paths taken from the S3 bucket.
///
/// Please beware that these `GefsVersion` numbers are entirely made up by us. They are not the
/// GEFS NWP model versions. Although there should be a simple mapping from our `GefsVersion`
/// numbers to the GEFS model version.
///
/// TODO: Extract the *actual* GEFS model version numbers from the GRIB files and use those as the
/// enum variant names.
#[derive(Debug, PartialEq, Clone)]
#[allow(non_camel_case_types)]
enum GefsVersion {
    /// GEFS model version 11?
    ///
    /// Paths of the form `gefs.20170101/00/gec00.t00z.pgrb2aanl.idx`
    V1,

    /// GEFS model version 11?
    ///
    /// Paths of the form `gefs.20180727/00/pgrb2[a|b]/gec00.t00z.pgrb2aanl.idx`
    V2,

    /// A union of the paths in V2 and V4 for just two runs! It looks like NOAA ran V2 and V3
    /// for two initialisations 2020-09-23T00 and T06. But the folders for these two
    /// init datetimes contain fewer files than in the equivalent "proper" V4 folders: e.g.:
    /// - gefs.20200923/00/atmos/pgrb2ap5 (V3) =  9,306 entries
    /// - gefs.20200924/00/atmos/pgrb2ap5 (V4) = 11,419 entries
    /// - gefs.20210101/00/atmos/pgrb2ap5 (V4) = 11,946 entries
    /// - gefs.20241010/00/atmos/pgrb2ap5 (V4) = 11,947 entries
    ///
    /// So it may be safest to ignore the "V4-like" folders in the folders for these
    /// two init datetimes and just use the "V2-like" folders for these two init times.
    /// i.e. just tread V3 as if it were V2.
    V3,

    /// GEFS model version v12?
    ///
    /// Paths of the forms:
    ///              This character-----v---v---v is repeated here-----v-v-v
    ///              These characters----vv--vv--vvv are repeated here---------v-vv-v
    /// - `gefs.20241008/00/atmos/pgrb2[ap5|bp5|sp25]/geavg.t00z.pgrb2[a|b|s].0p[25|50].f000.idx`
    /// - `gefs.20241008/00/atmos/[bufr|init]/`: Ignore! No GRIB data.
    /// - `gefs.20241008/00/chem/[pgrb2ap25|pgrb2ap5]/gefs.chem.t00z.a2d_0p25.f000.grib2.idx`
    /// - `gefs.20241008/00/wave/`
    ///     - `gridded/gefs.wave.t00z.c00.global.0p25.f000.grib2.idx`
    ///     - `station`: Ignore! No GRIB data!
    V4,
}

impl GefsVersion {
    const N_GFS_VERSIONS: usize = 4;
    const ALL_GEFS_VERSIONS: [Self; Self::N_GFS_VERSIONS] =
        [Self::V1, Self::V2, Self::V3, Self::V4];

    /// This is the reference datetime at which this version becomes active. Each version lasts
    /// until the next version's start_reference_datetime minus 6 hours.
    fn start_reference_datetime(&self) -> DateTime<Utc> {
        match *self {
            Self::V1 => ymdh_to_datetime(2017, 1, 1, 0),
            Self::V2 => ymdh_to_datetime(2018, 7, 27, 0),
            Self::V3 => ymdh_to_datetime(2020, 9, 23, 0),
            Self::V4 => ymdh_to_datetime(2020, 9, 23, 12),
        }
    }

    fn try_from_reference_datetime(
        query_datetime: &DateTime<Utc>,
    ) -> Result<&'static Self, BeforeStartOfDatasetError> {
        for i in 0..Self::N_GFS_VERSIONS - 1 {
            let this_gfs_version = &Self::ALL_GEFS_VERSIONS[i];
            let next_gfs_version = &Self::ALL_GEFS_VERSIONS[i + 1];
            if *query_datetime >= this_gfs_version.start_reference_datetime()
                && *query_datetime < next_gfs_version.start_reference_datetime()
            {
                return Ok(this_gfs_version);
            }
        }
        let last_gfs_version = &Self::ALL_GEFS_VERSIONS[Self::N_GFS_VERSIONS - 1];
        if *query_datetime >= last_gfs_version.start_reference_datetime() {
            Ok(last_gfs_version)
        } else {
            // The `query_datetime` is before the start of the dataset!
            Err(BeforeStartOfDatasetError)
        }
    }
}

#[derive(Debug)]
struct BeforeStartOfDatasetError;

struct GefsIdxPath<'a> {
    path: &'a object_store::path::Path,
    parts: Vec<object_store::path::PathPart<'a>>,
}

impl<'a> TryFrom<&'a object_store::path::Path> for GefsIdxPath<'a> {
    type Error = GefsIdxError;

    fn try_from(idx_path: &'a object_store::path::Path) -> Result<Self, Self::Error> {
        let parts: Vec<_> = idx_path.parts().collect();

        // Helper closure:
        let make_error = |error: String| -> Result<Self, Self::Error> {
            Err(GefsIdxError {
                error,
                path: idx_path.to_string(),
            })
        };

        // Sanity checks:
        if parts[0].as_ref() != BUCKET_ID {
            return make_error(format!("GEFS path must start with '{BUCKET_ID}'."));
        }

        // Check the number of parts:
        let n_parts = parts.len();
        const N_PARTS_EXPECTED: [usize; 3] = [4, 5, 6];
        if !N_PARTS_EXPECTED.contains(&n_parts) {
            return make_error(format!(
                "Expected {N_PARTS_EXPECTED:?} parts in the path of the idx file. Found {n_parts} parts instead.",
            ));
        }

        // Check that the path ends with `.idx`.
        let last_part = &parts[n_parts - 1];
        if !last_part.as_ref().ends_with(".idx") {
            return make_error("The path must end with '.idx'!".to_string());
        }

        Ok(Self {
            path: idx_path,
            parts,
        })
    }
}

impl GefsIdxPath<'_> {
    fn extract_reference_datetime(&self) -> anyhow::Result<DateTime<Utc>> {
        let date = NaiveDate::parse_from_str(self.parts[1].as_ref(), "gefs.%Y%m%d")?;
        let hour: u32 = self.parts[2].as_ref().parse()?;
        match date.and_hms_opt(hour, 0, 0) {
            Some(dt) => Ok(dt.and_utc()),
            None => Err(GefsIdxError {
                error: format!("Invalid hour {hour} when parsing path of idx"),
                path: self.path.to_string(),
            }
            .into()),
        }
    }
}

fn ymdh_to_datetime(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
    match Utc.with_ymd_and_hms(year, month, day, hour, 0, 0) {
        chrono::offset::LocalResult::Single(dt) => dt,
        _ => panic!("Invalid datetime! {year}-{month}-{day}T{hour}"),
    }
}

#[cfg(test)]
mod tests {

    use std::{cell::OnceCell, sync::OnceLock};

    use chrono::{naive::serde::ts_milliseconds_option::deserialize, NaiveDateTime};

    use crate::ToIdxPath;

    use super::*;

    #[derive(PartialEq, Debug, serde::Deserialize, Clone)]
    struct GfsTest {
        path: String,
        #[serde(deserialize_with = "deserialize_gefs_version_enum")]
        gefs_version_enum_variant: GefsVersion,
        #[serde(deserialize_with = "deserialize_reference_datetime")]
        reference_datetime: DateTime<Utc>,
        ensemble_member: String,
        #[serde(deserialize_with = "deserialize_forecast_hour")]
        forecast_hour: TimeDelta,
    }

    fn deserialize_reference_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        let s = format!("{s}00");
        match NaiveDateTime::parse_from_str(&s, "%Y%m%dT%H%M") {
            Ok(dt) => Ok(dt.and_utc()),
            Err(e) => Err(serde::de::Error::custom(format!(
                "Invalid init datetime: {e}"
            ))),
        }
    }

    fn deserialize_gefs_version_enum<'de, D>(deserializer: D) -> Result<GefsVersion, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let variant_i = <usize>::deserialize(deserializer)?;
        Ok(GefsVersion::ALL_GEFS_VERSIONS[variant_i].clone())
    }

    fn deserialize_forecast_hour<'de, D>(deserializer: D) -> Result<TimeDelta, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let forecast_hour = <i64>::deserialize(deserializer)?;
        Ok(TimeDelta::hours(forecast_hour))
    }

    fn load_gefs_test_paths_csv() -> Vec<GfsTest> {
        // Gets the MANIFEST_DIR of the sub-crate.
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("src/datasets/gefs_test_paths.csv");
        let mut rdr =
            csv::Reader::from_path(&d).expect(format!("Failed to open {:?}", &d).as_str());
        let mut records = vec![];
        for result in rdr.deserialize() {
            records.push(result.unwrap());
        }
        records
    }

    static GEFS_TEST_DATA: OnceLock<Vec<GfsTest>> = OnceLock::new();

    #[test]
    fn test_idx_path_to_reference_datetime() {
        GEFS_TEST_DATA
            .get_or_init(move || load_gefs_test_paths_csv())
            .iter()
            .for_each(|gefs_test_struct| {
                let path = object_store::path::Path::try_from(gefs_test_struct.path.as_str())
                    .expect(&format!(
                        "Failed to parse path string into an object_store::path::Path! {}",
                        gefs_test_struct.path
                    ));
                let dt = GefsIdxPath::try_from(&path)
                    .unwrap()
                    .extract_reference_datetime()
                    .expect(&format!(
                        "Failed to extract reference datetime from {}",
                        gefs_test_struct.path
                    ));
                assert_eq!(
                    dt, gefs_test_struct.reference_datetime,
                    "Incorrect reference datetime when parsing idx path '{path}'"
                );
            });
    }

    #[test]
    fn test_try_from_reference_datetime() {
        assert!(
            GefsVersion::try_from_reference_datetime(&ymdh_to_datetime(2000, 1, 1, 0)).is_err()
        );
        GEFS_TEST_DATA
            .get_or_init(move || load_gefs_test_paths_csv())
            .iter()
            .for_each(|gefs_test_struct| {
                assert_eq!(
                    GefsVersion::try_from_reference_datetime(&gefs_test_struct.reference_datetime)
                        .unwrap(),
                    &gefs_test_struct.gefs_version_enum_variant,
                )
            });
    }

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
