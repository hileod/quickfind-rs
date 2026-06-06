use std::collections::HashMap;

use super::{FileId, ParentFileId};

#[derive(Debug, Default)]
pub struct PathResolver {
    nodes: HashMap<FileId, PathNode>,
}

#[derive(Debug, Clone)]
struct PathNode {
    parent_id: Option<ParentFileId>,
    name: String,
}

impl PathResolver {
    pub fn upsert(&mut self, file_id: FileId, parent_id: Option<ParentFileId>, name: String) {
        self.nodes.insert(file_id, PathNode { parent_id, name });
    }

    pub fn remove(&mut self, file_id: FileId) {
        self.nodes.remove(&file_id);
    }

    pub fn resolve(&self, file_id: FileId) -> Option<String> {
        let mut parts = Vec::new();
        let mut current = file_id;

        loop {
            let node = self.nodes.get(&current)?;
            parts.push(node.name.as_str());
            let Some(parent_id) = node.parent_id else {
                break;
            };
            current = FileId(parent_id.0);
        }

        parts.reverse();
        Some(parts.join("\\"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_path_from_file_and_parent_ids() {
        let mut resolver = PathResolver::default();
        resolver.upsert(FileId(1), None, String::from("C:"));
        resolver.upsert(FileId(2), Some(ParentFileId(1)), String::from("Users"));
        resolver.upsert(FileId(3), Some(ParentFileId(2)), String::from("report.txt"));

        assert_eq!(
            resolver.resolve(FileId(3)).as_deref(),
            Some(r"C:\Users\report.txt")
        );
    }
}
