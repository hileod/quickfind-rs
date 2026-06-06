pub mod index_store;
pub mod mft_reader;
pub mod path_resolver;
pub mod service;
pub mod usn_watcher;
pub mod volume_scanner;

use crate::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VolumeInfo {
    pub root: String,
    pub name: String,
    pub file_system: FileSystemKind,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FileSystemKind {
    Ntfs,
    Refs,
    Other,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct FileId(pub u64);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ParentFileId(pub u64);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UsnChangeKind {
    Created,
    Deleted,
    Renamed,
    Modified,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UsnChange {
    pub file_id: FileId,
    pub parent_id: ParentFileId,
    pub name: String,
    pub kind: UsnChangeKind,
}

pub fn is_supported_volume(file_system: FileSystemKind) -> bool {
    matches!(file_system, FileSystemKind::Ntfs | FileSystemKind::Refs)
}

pub(crate) fn unsupported(feature: &str) -> crate::Result<()> {
    Err(format!("{feature} is not implemented yet").into())
}

pub fn bootstrap_windows_indexer() -> Result<()> {
    unsupported("Windows service + MFT/USN indexer bootstrap")
}
