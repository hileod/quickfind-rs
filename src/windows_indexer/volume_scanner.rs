use super::{FileSystemKind, VolumeInfo};
use crate::Result;

pub fn enumerate_volumes() -> Result<Vec<VolumeInfo>> {
    #[cfg(windows)]
    {
        // TODO: Replace with FindFirstVolumeW / GetVolumeInformationW so this
        // can detect NTFS/ReFS volumes without relying on drive letters.
        super::unsupported("NTFS/ReFS volume enumeration")?;
        unreachable!()
    }

    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

pub fn parse_file_system_name(name: &str) -> FileSystemKind {
    match name.to_ascii_uppercase().as_str() {
        "NTFS" => FileSystemKind::Ntfs,
        "REFS" => FileSystemKind::Refs,
        _ => FileSystemKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_file_systems() {
        assert_eq!(parse_file_system_name("NTFS"), FileSystemKind::Ntfs);
        assert_eq!(parse_file_system_name("refs"), FileSystemKind::Refs);
        assert_eq!(parse_file_system_name("FAT32"), FileSystemKind::Other);
    }
}
