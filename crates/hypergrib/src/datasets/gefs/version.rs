use chrono::{DateTime, Utc};

use crate::ymdh_to_datetime;

/// The structure of the GEFS paths has changed over time.
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
pub(crate) enum Version {
    /// GEFS model version 11?
    ///
    /// Paths of the form `gefs.20170101/00/gec00.t00z.pgrb2aanl.idx`
    V0,

    /// GEFS model version 11?
    ///
    /// Paths of the form `gefs.20180727/00/pgrb2[a|b]/gec00.t00z.pgrb2aanl.idx`
    V1,

    /// A union of the paths in V1 and V3 for just two runs! It looks like NOAA ran V1 and V3
    /// for two initialisations 2020-09-23T00 and T06. But the folders for these two
    /// init datetimes contain fewer files than in the equivalent "proper" V3 folders: e.g.:
    /// - gefs.20200923/00/atmos/pgrb2ap5 (V2) =  9,306 entries
    /// - gefs.20200924/00/atmos/pgrb2ap5 (V3) = 11,419 entries
    /// - gefs.20210101/00/atmos/pgrb2ap5 (V3) = 11,946 entries
    /// - gefs.20241010/00/atmos/pgrb2ap5 (V3) = 11,947 entries
    ///
    /// So it may be safest to ignore the "V3-like" folders in the folders for these
    /// two init datetimes and just use the "V1-like" folders for these two init times.
    /// i.e. just tread V2 as if it were V1.
    V2,

    /// GEFS model version v12?
    ///
    /// Paths of the forms:
    /// ```text
    ///             This character-----v---v---v is repeated here-----v-v-v
    ///             These characters----vv--vv--vvv are repeated here---------v-vv-v
    /// - gefs.20241008/00/atmos/pgrb2[ap5|bp5|sp25]/geavg.t00z.pgrb2[a|b|s].0p[25|50].f000.idx
    /// - gefs.20241008/00/atmos/[bufr|init]/: Ignore! No GRIB data.
    /// - gefs.20241008/00/chem/[pgrb2ap25|pgrb2ap5]/gefs.chem.t00z.a2d_0p25.f000.grib2.idx
    /// - gefs.20241008/00/wave/`
    ///     - gridded/gefs.wave.t00z.c00.global.0p25.f000.grib2.idx
    ///     - station: Ignore! No GRIB data!
    /// ```
    V3,
}

impl Version {
    const N_VERSIONS: usize = 4;
    const ALL_VERSIONS: [Self; Self::N_VERSIONS] = [Self::V0, Self::V1, Self::V2, Self::V3];

    /// This is the reference datetime at which this version becomes active. Each version lasts
    /// until the next version's start_reference_datetime minus 6 hours.
    fn start_reference_datetime(&self) -> DateTime<Utc> {
        match *self {
            Self::V0 => ymdh_to_datetime(2017, 1, 1, 0),
            Self::V1 => ymdh_to_datetime(2018, 7, 27, 0),
            Self::V2 => ymdh_to_datetime(2020, 9, 23, 0),
            Self::V3 => ymdh_to_datetime(2020, 9, 23, 12),
        }
    }

    fn try_from_reference_datetime(
        query_datetime: &DateTime<Utc>,
    ) -> Result<&'static Self, BeforeStartOfDatasetError> {
        for i in 0..Self::N_VERSIONS - 1 {
            let this_gfs_version = &Self::ALL_VERSIONS[i];
            let next_gfs_version = &Self::ALL_VERSIONS[i + 1];
            if *query_datetime >= this_gfs_version.start_reference_datetime()
                && *query_datetime < next_gfs_version.start_reference_datetime()
            {
                return Ok(this_gfs_version);
            }
        }
        let last_gfs_version = &Self::ALL_VERSIONS[Self::N_VERSIONS - 1];
        if *query_datetime >= last_gfs_version.start_reference_datetime() {
            Ok(last_gfs_version)
        } else {
            // The `query_datetime` is before the start of the dataset!
            Err(BeforeStartOfDatasetError)
        }
    }

    pub(crate) const fn all_versions() -> [Self; Self::N_VERSIONS] {
        Self::ALL_VERSIONS
    }
}

#[derive(Debug)]
struct BeforeStartOfDatasetError;

#[cfg(test)]
mod tests {

    use super::*;
    use crate::datasets::gefs::test_utils::load_gefs_test_paths_csv;

    #[test]
    fn test_try_from_reference_datetime() {
        assert!(Version::try_from_reference_datetime(&ymdh_to_datetime(2000, 1, 1, 0)).is_err());
        load_gefs_test_paths_csv()
            .iter()
            .for_each(|gefs_test_struct| {
                assert_eq!(
                    Version::try_from_reference_datetime(&gefs_test_struct.reference_datetime)
                        .unwrap(),
                    &gefs_test_struct.gefs_version_enum_variant,
                )
            });
    }
}
