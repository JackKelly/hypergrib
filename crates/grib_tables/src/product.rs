use crate::{
    category::{Category, HydrologicalCategory, MeteorologicalCategory},
    center_and_table_versions::CenterAndTableVersions,
    master_table::MasterTable,
    originating_center::OriginatingCenter,
};

pub(crate) enum Product {
    Meteorological(MeteorologicalCategory),
    Hydrological(HydrologicalCategory),

    // Local to NCEP:
    NcepFoo(NcepFooProduct),
    // etc.
}

impl Product {
    pub fn from_discipline_and_category_and_parameter_numbers(
        discipline_num: u8,
        category_num: u8,
        parameter_num: u8,
        center_and_table_versions: CenterAndTableVersions,
    ) -> Option<Product> {
        // This function just routes the query to the functions which handle Disciplines specified
        // in either local or master tables.
        match discipline_num {
            ..192 => Product::from_master_discipline_and_category_and_parameter_numbers(
                discipline_num,
                category_num,
                parameter_num,
                center_and_table_versions,
            ),

            // Reserved for local use:
            192..=254 => Product::from_local_discipline_and_category_and_parameter_numbers(
                discipline_num,
                category_num,
                parameter_num,
                center_and_table_versions,
            ),

            255 => None, // 255 means "missing"
        }
    }

    fn from_master_discipline_and_category_and_parameter_numbers(
        discipline_num: u8,
        category_num: u8,
        parameter_num: u8,
        center_and_table_versions: CenterAndTableVersions,
    ) -> Option<Self> {
        match discipline_num {
            0 => Some(Product::Meteorological(
                MeteorologicalCategory::from_category_and_parameter_numbers(
                    category_num,
                    parameter_num,
                    center_and_table_versions,
                )?,
            )),

            // Demo of how to handle a discipline number which changes meaning across different
            // master table versions. This discipline number is made up! Just for demo purposes!
            191 => match center_and_table_versions.master_table {
                MasterTable::V32 => todo!(),
                MasterTable::V33 => todo!(),
            },

            // Reserved for local use:
            192..=254 => panic!("Local disciplines should never be passed to this function!"),
            _ => None,
        }
    }

    fn from_local_discipline_and_category_and_parameter_numbers(
        discipline_num: u8,
        category_num: u8,
        parameter_num: u8,
        center_and_table_versions: CenterAndTableVersions,
    ) -> Option<Product> {
        match center_and_table_versions.originating_center {
            OriginatingCenter::NCEP => match discipline_num {
                192 => Some(Product::NcepFoo(
                    NcepFooProduct::from_category_and_parameter_numbers(
                        category_num,
                        parameter_num,
                        center_and_table_versions,
                    )?,
                )),
            },
        }
    }
}
