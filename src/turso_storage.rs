use std::fs;
use std::path::{Path, PathBuf};

use tokio::runtime::Builder as RuntimeBuilder;
use turso::{Builder, params};

use crate::Result;
use crate::index::{EntryKind, FileEntry};

pub fn is_turso_metadata(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("db" | "sqlite" | "sqlite3")
    )
}

pub fn metadata_path_for(index_path: &Path) -> PathBuf {
    let stem = index_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("index");
    let metadata_name = format!("{stem}.meta.db");

    index_path
        .parent()
        .map(|parent| parent.join(&metadata_name))
        .unwrap_or_else(|| PathBuf::from(metadata_name))
}

pub fn write_metadata(path: &Path, entries: &[FileEntry]) -> Result<()> {
    run_async(write_metadata_async(path, entries))
}

pub fn read_metadata(path: &Path) -> Result<Vec<FileEntry>> {
    run_async(read_metadata_async(path))
}

async fn write_metadata_async(path: &Path, entries: &[FileEntry]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let db = Builder::new_local(path.to_string_lossy().as_ref())
        .build()
        .await?;
    let conn = db.connect()?;
    conn.execute_batch(
        "\
CREATE TABLE IF NOT EXISTS files (
    path TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL DEFAULT 'file',
    name TEXT NOT NULL,
    lower_path TEXT NOT NULL,
    lower_name TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_files_lower_name ON files(lower_name);
CREATE INDEX IF NOT EXISTS idx_files_lower_path ON files(lower_path);
",
    )
    .await?;
    let _ = conn
        .execute_batch("ALTER TABLE files ADD COLUMN kind TEXT NOT NULL DEFAULT 'file';")
        .await;

    conn.execute_batch("BEGIN IMMEDIATE; DELETE FROM files;")
        .await?;
    let mut statement = conn
        .prepare(
            "\
INSERT INTO files (path, kind, name, lower_path, lower_name)
VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(path) DO UPDATE SET
    kind = excluded.kind,
    name = excluded.name,
    lower_path = excluded.lower_path,
    lower_name = excluded.lower_name
",
        )
        .await?;

    for entry in entries {
        statement
            .execute(params![
                entry.path.as_str(),
                entry.kind.as_str(),
                entry.name(),
                entry.lower_path.as_str(),
                entry.lower_name.as_str(),
            ])
            .await?;
    }
    conn.execute_batch("COMMIT;").await?;
    conn.cacheflush()?;

    Ok(())
}

async fn read_metadata_async(path: &Path) -> Result<Vec<FileEntry>> {
    let db = Builder::new_local(path.to_string_lossy().as_ref())
        .build()
        .await?;
    let conn = db.connect()?;
    let mut rows = conn
        .query("SELECT path, kind FROM files ORDER BY lower_path", ())
        .await?;
    let mut entries = Vec::new();

    while let Some(row) = rows.next().await? {
        let path = row.get::<String>(0)?;
        let kind = EntryKind::from_str(&row.get::<String>(1)?);
        entries.push(FileEntry::from_string_with_kind(path, kind));
    }

    Ok(entries)
}

fn run_async<T>(future: impl Future<Output = Result<T>>) -> Result<T> {
    RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn detects_turso_metadata_extensions() {
        assert!(is_turso_metadata(Path::new("index.db")));
        assert!(is_turso_metadata(Path::new("index.sqlite")));
        assert!(is_turso_metadata(Path::new("index.sqlite3")));
        assert!(!is_turso_metadata(Path::new("index.qf")));
    }

    #[test]
    fn derives_metadata_path_from_index_path() {
        assert_eq!(
            metadata_path_for(Path::new(r"C:\data\index.qf")),
            PathBuf::from(r"C:\data\index.meta.db")
        );
    }

    #[test]
    fn round_trips_turso_metadata() {
        let dir = std::env::temp_dir().join(format!(
            "quickfind-turso-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("index.db");
        let entries = vec![
            FileEntry::from_string(r"C:\Work\Cargo.toml".to_string()),
            FileEntry::from_string_with_kind(r"C:\Work\src".to_string(), EntryKind::Directory),
        ];

        write_metadata(&path, &entries).unwrap();
        let loaded = read_metadata(&path).unwrap();

        assert_eq!(loaded, entries);
        let _ = fs::remove_file(path);
        let _ = fs::remove_dir(dir);
    }
}
