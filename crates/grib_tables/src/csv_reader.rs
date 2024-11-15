pub(crate) mod read_local_index;
pub(crate) mod read_table_4_2;

use std::path::PathBuf;

fn csv_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_dir = PathBuf::from(manifest_dir);
    manifest_dir.join("csv")
}
