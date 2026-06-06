use std::time::Duration;

use crate::Result;

use super::{UsnChange, VolumeInfo};

pub struct UsnWatcher {
    volume: VolumeInfo,
}

impl UsnWatcher {
    pub fn new(volume: VolumeInfo) -> Self {
        Self { volume }
    }

    pub fn poll_changes(&mut self, wait: Duration) -> Result<Vec<UsnChange>> {
        let _ = wait;
        let _ = &self.volume;
        super::unsupported("USN Journal watcher")?;
        unreachable!()
    }
}
