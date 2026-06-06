use crate::Result;
use crate::index::FileEntry;

use super::VolumeInfo;

#[derive(Debug, Clone)]
pub struct MftSnapshot {
    pub volume: VolumeInfo,
    pub entries: Vec<FileEntry>,
}

pub fn read_initial_snapshot(volume: &VolumeInfo) -> Result<MftSnapshot> {
    let _ = volume;
    super::unsupported("MFT initial snapshot reader")?;
    unreachable!()
}
