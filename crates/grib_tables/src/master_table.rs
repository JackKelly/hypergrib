use crate::{
    category::meteorological::{temperature::TemperatureParameter, MeteorologicalCategory},
    product::Product,
    AbbrevToProduct,
};

pub(crate) enum MasterTable {
    V32,
    V33,
}

impl AbbrevToProduct for MasterTable {
    fn abbrev_to_product(&self, abbrev: &str) -> Option<&'static Product> {
        MasterTable::abbrev_to_product_master_table_common()
            .get(abbrev)
            .or_else(|| {
                match self {
                    MasterTable::V32 => Self::abbrev_to_product_master_table_v32(),
                    _ => todo!(),
                }
                .get(abbrev)
            })
    }
}

impl MasterTable {
    fn abbrev_to_product_master_table_common() -> &'static phf::Map<&'static str, Product> {
        /// All the abbreviations which are common across all centers and all table versions.
        ///
        /// To decode .idx files, we need a single hashmap which holds every abbreviation string.
        /// So the values of the hashmap have to all be the same type.
        ///
        /// `phf::Map` is compiled to a perfect hash table, which is O(1). In contrast,
        /// matching strings compiles code which checks each string in turn, which is O(n).
        static ABBREV_TO_PRODUCT_COMMON: phf::Map<&'static str, Product> = phf::phf_map! {
            "TMP" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::Temperature)),
            "VTMP" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::VirtualTemperature)),
            "POT" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::PotentialTemperature)),
            "EPOT" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::PseudoAdiabaticPotentialTemperature)),
            "TMAX" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::MaximumTemperature)),
            "TMIN" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::MinimumTemperature)),
            "DPT" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::DewPointTemperature)),
            "DEPR" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::DewPointDepression)),
            "LAPR" => Product::Meteorological(MeteorologicalCategory::Temperature(TemperatureParameter::LapseRate)),
        };
        &ABBREV_TO_PRODUCT_COMMON
    }

    fn abbrev_to_product_master_table_v32() -> &'static phf::Map<&'static str, Product> {
        // Contains only the diff between master table V32 and the common abbreviations.
        static ABBREV_TO_PRODUCT_MASTER_TABLE_V32: phf::Map<&'static str, Product> =
            phf::phf_map! {}; // TODO: Fill in this map!
        &ABBREV_TO_PRODUCT_MASTER_TABLE_V32
    }
}
