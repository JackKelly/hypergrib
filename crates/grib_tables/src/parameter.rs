mod database;
mod numeric_id;

#[derive(Clone, Debug, derive_more::Display, PartialEq, Eq)]
#[display("({}, {}, {}, {}, {})", description, note, unit, abbreviation, status)]
pub(crate) struct Parameter {
    pub(crate) description: String,
    pub(crate) note: String,
    pub(crate) unit: String, // TODO: Maybe use a Unit enum? Or load units from wmo-im/CCT/C06.csv?
    pub(crate) abbreviation: Abbreviation,
    pub(crate) status: Status,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, derive_more::Display)]
pub(crate) struct Abbreviation(pub(crate) String);

#[derive(Clone, Debug, derive_more::Display, PartialEq, Eq)]
pub(crate) enum Status {
    Operational,
    Deprecated,
}
