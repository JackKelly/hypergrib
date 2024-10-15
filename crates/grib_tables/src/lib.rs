use center_and_table_versions::CenterAndTableVersions;

pub(crate) mod category;
pub(crate) mod center_and_table_versions;
pub(crate) mod master_table;
pub(crate) mod originating_center;
pub(crate) mod product;

pub trait AbbrevToProduct {
    fn abbrev_to_product(&self, abbrev: &str) -> Option<&'static crate::product::Product>;
}

trait Parameter {
    fn from_parameter_num(
        parameter_num: u8,
        center_and_table_versions: CenterAndTableVersions,
    ) -> Option<Self>
    where
        Self: Sized;

    fn abbrev(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn unit(&self) -> &'static str;
}
