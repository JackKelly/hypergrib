mod database;
mod numeric_id;

#[derive(Clone, Debug, derive_more::Display, PartialEq, Eq)]
#[display("({}, {}, {})", short_name, name, unit)]
pub(crate) struct Parameter {
    pub(crate) short_name: ShortName,
    pub(crate) name: String,
    pub(crate) unit: String, // TODO: Maybe use a Unit enum?
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, derive_more::Display)]
pub(crate) struct ShortName(pub(crate) String);
