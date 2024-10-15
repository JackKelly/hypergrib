use crate::{
    master_table::MasterTable, originating_center::OriginatingCenter, product::Product,
    AbbrevToProduct,
};

pub(crate) struct CenterAndTableVersions {
    originating_center: OriginatingCenter,
    local_table_version: u8,
    master_table: MasterTable,
}

impl AbbrevToProduct for CenterAndTableVersions {
    fn abbrev_to_product(&self, abbrev: &str) -> Option<&'static Product> {
        self.master_table
            .abbrev_to_product(abbrev)
            .or_else(|| self.originating_center.abbrev_to_product(abbrev))
    }
}

impl CenterAndTableVersions {
    pub(crate) fn originating_center(&self) -> &OriginatingCenter {
        &self.originating_center
    }
}
