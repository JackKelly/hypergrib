use crate::category::meteorological::temperature::TemperatureParameter;
use crate::category::meteorological::MeteorologicalCategory;
use crate::product::Product;
use crate::AbbrevToProduct;

/// Identification of originating/generating center.
pub(crate) enum OriginatingCenter {
    NCEP { local_table_version: u8 },
}

impl AbbrevToProduct for OriginatingCenter {
    fn abbrev_to_product(&self, abbrev: &str) -> Option<&'static Product> {
        match self {
            OriginatingCenter::NCEP {
                local_table_version,
            } => OriginatingCenter::abbrev_to_product_ncep(*local_table_version),
        }
        .get(abbrev)
    }
}

impl OriginatingCenter {
    fn abbrev_to_product_ncep(
        _local_table_version: u8,
    ) -> &'static phf::Map<&'static str, Product> {
        static ABBREV_TO_PRODUCT_NCEP: phf::Map<&'static str, Product> = phf::phf_map! {
            "SNOHF" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::NcepSnowPhaseChangeHeatFlux)),
            "TTRAD" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::NcepTemperatureTendencyByAllRadiation)),
        };
        &ABBREV_TO_PRODUCT_NCEP
    }
}
