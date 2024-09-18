#![doc = include_str!("../README.md")]

pub mod datasets;

pub(crate) trait Key: TryFrom<object_store::path::Path> {
    fn to_path(&self) -> object_store::path::Path;
}
