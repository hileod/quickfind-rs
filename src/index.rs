use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "folder",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "folder" | "directory" => Self::Directory,
            _ => Self::File,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Self::File => 0,
            Self::Directory => 1,
        }
    }

    pub fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Directory,
            _ => Self::File,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FileEntry {
    pub path: String,
    pub kind: EntryKind,
    pub name_start: usize,
    pub lower_path: String,
    pub lower_name: String,
}

impl FileEntry {
    pub fn from_path(path: PathBuf) -> Self {
        Self::from_path_with_kind(path, EntryKind::File)
    }

    pub fn from_path_with_kind(path: PathBuf, kind: EntryKind) -> Self {
        let path = path.to_string_lossy().into_owned();
        Self::from_string_with_kind(path, kind)
    }

    pub fn from_string(path: String) -> Self {
        Self::from_string_with_kind(path, EntryKind::File)
    }

    pub fn from_string_with_kind(path: String, kind: EntryKind) -> Self {
        let name_start = path
            .rfind(['\\', '/'])
            .map(|index| index + 1)
            .unwrap_or_default();
        let lower_path = path.to_lowercase();
        let lower_name = path[name_start..].to_lowercase();

        Self {
            path,
            kind,
            name_start,
            lower_path,
            lower_name,
        }
    }

    pub fn name(&self) -> &str {
        &self.path[self.name_start..]
    }
}

pub fn sort_and_dedup(entries: &mut Vec<FileEntry>) {
    entries.sort_unstable_by(|left, right| left.lower_path.cmp(&right.lower_path));
    entries.dedup_by(|left, right| left.lower_path == right.lower_path);
}
