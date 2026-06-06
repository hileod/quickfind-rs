use crate::index::{EntryKind, FileEntry};

use super::SearchFilters;

#[derive(Debug, Clone)]
pub struct SearchDocument<'a> {
    pub entry: &'a FileEntry,
    pub name: &'a str,
    pub lower_name: &'a str,
    pub lower_path: &'a str,
    pub extension: Option<String>,
    pub drive: Option<String>,
}

impl<'a> SearchDocument<'a> {
    pub fn from_entry(entry: &'a FileEntry) -> Self {
        Self {
            entry,
            name: entry.name(),
            lower_name: &entry.lower_name,
            lower_path: &entry.lower_path,
            extension: extension(entry.name()),
            drive: drive(&entry.path),
        }
    }

    pub fn matches_filters(&self, filters: &SearchFilters) -> bool {
        if let Some(kind) = filters.kind
            && self.entry.kind != kind
        {
            return false;
        }

        if let Some(extension) = filters.extension.as_deref()
            && self.extension.as_deref() != Some(extension)
        {
            return false;
        }

        if let Some(drive) = filters.drive.as_deref()
            && self.drive.as_deref() != Some(drive)
        {
            return false;
        }

        true
    }

    pub fn is_folder(&self) -> bool {
        self.entry.kind == EntryKind::Directory
    }
}

fn extension(name: &str) -> Option<String> {
    name.rsplit_once('.')
        .and_then(|(_, extension)| (!extension.is_empty()).then(|| extension.to_lowercase()))
}

fn drive(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        Some(path[..2].to_ascii_uppercase())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::index::{EntryKind, FileEntry};

    use super::*;

    #[test]
    fn extracts_extension_and_drive() {
        let entry = FileEntry::from_string(r"C:\Users\liam\report.PDF".to_string());
        let document = SearchDocument::from_entry(&entry);

        assert_eq!(document.extension.as_deref(), Some("pdf"));
        assert_eq!(document.drive.as_deref(), Some("C:"));
    }

    #[test]
    fn filters_by_kind() {
        let entry = FileEntry::from_string_with_kind(
            r"C:\Users\liam\docs".to_string(),
            EntryKind::Directory,
        );
        let document = SearchDocument::from_entry(&entry);
        let filters = SearchFilters {
            kind: Some(EntryKind::Directory),
            ..SearchFilters::default()
        };

        assert!(document.matches_filters(&filters));
    }
}
