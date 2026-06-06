use std::path::Path;

use crate::Result;
use crate::index::FileEntry;
use crate::storage;
use crate::turso_storage;

pub struct HybridIndexStore;

impl HybridIndexStore {
    pub fn write_snapshot(index_path: &Path, entries: &[FileEntry]) -> Result<()> {
        storage::write_index(index_path, entries)?;
        let metadata_path = turso_storage::metadata_path_for(index_path);
        turso_storage::write_metadata(&metadata_path, entries)?;
        Ok(())
    }

    pub fn read_hot_index(index_path: &Path) -> Result<Vec<FileEntry>> {
        storage::read_index(index_path)
    }
}
