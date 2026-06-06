use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use crate::Result;
use crate::index::{EntryKind, FileEntry};

const MAGIC: &[u8; 8] = b"QFIDX1\0\0";
const VERSION: u32 = 2;
const LEGACY_VERSION: u32 = 1;

pub fn write_index(path: &Path, entries: &[FileEntry]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(MAGIC)?;
    writer.write_all(&VERSION.to_le_bytes())?;
    writer.write_all(&(entries.len() as u64).to_le_bytes())?;

    for entry in entries {
        let bytes = entry.path.as_bytes();
        let len = u32::try_from(bytes.len()).map_err(|_| "path too long for index format")?;
        writer.write_all(&[entry.kind.to_byte()])?;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(bytes)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn read_index(path: &Path) -> Result<Vec<FileEntry>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut magic = [0; 8];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(format!("{} is not a quickfind index", path.display()).into());
    }

    let version = read_u32(&mut reader)?;
    if version != VERSION && version != LEGACY_VERSION {
        return Err(format!("unsupported index version: {version}").into());
    }

    let count = read_u64(&mut reader)?;
    let mut entries = Vec::with_capacity(count.min(usize::MAX as u64) as usize);

    for _ in 0..count {
        let kind = if version >= VERSION {
            let mut kind = [0; 1];
            reader.read_exact(&mut kind)?;
            EntryKind::from_byte(kind[0])
        } else {
            EntryKind::File
        };
        let len = read_u32(&mut reader)? as usize;
        let mut bytes = vec![0; len];
        reader.read_exact(&mut bytes)?;
        let path = String::from_utf8(bytes)?;
        entries.push(FileEntry::from_path_with_kind(PathBuf::from(path), kind));
    }

    Ok(entries)
}

fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> io::Result<u64> {
    let mut bytes = [0; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn round_trips_index() {
        let dir = std::env::temp_dir().join(format!("quickfind-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("index.qf");
        let entries = vec![
            FileEntry::from_string(r"C:\Work\Cargo.toml".to_string()),
            FileEntry::from_string_with_kind(r"C:\Work\src".to_string(), EntryKind::Directory),
            FileEntry::from_string_with_kind(
                r"C:\Program Files\Quickfind\Quickfind.exe".to_string(),
                EntryKind::Application,
            ),
        ];

        write_index(&path, &entries).unwrap();
        let loaded = read_index(&path).unwrap();

        assert_eq!(loaded, entries);
        let _ = fs::remove_file(path);
        let _ = fs::remove_dir(dir);
    }
}
