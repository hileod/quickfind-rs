use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crate::Result;
use crate::index::{EntryKind, FileEntry, sort_and_dedup};

#[derive(Debug, Default)]
pub struct ScanReport {
    pub entries: Vec<FileEntry>,
    pub skipped_dirs: usize,
    pub skipped_items: usize,
}

pub fn build_index(roots: Vec<PathBuf>, thread_count: usize) -> Result<Vec<FileEntry>> {
    Ok(build_index_with_report(roots, thread_count)?.entries)
}

pub fn build_index_with_report(roots: Vec<PathBuf>, thread_count: usize) -> Result<ScanReport> {
    let queue = Arc::new(WorkQueue::new(roots));
    let (sender, receiver) = mpsc::channel();
    let mut workers = Vec::new();

    for _ in 0..thread_count {
        let queue = Arc::clone(&queue);
        let sender = sender.clone();
        workers.push(thread::spawn(move || {
            let mut batch = Vec::with_capacity(4096);
            let mut skipped_dirs = 0;
            let mut skipped_items = 0;
            while let Some(dir) = queue.pop() {
                if dir.is_file() {
                    let kind = file_kind(&dir);
                    batch.push(FileEntry::from_path_with_kind(dir, kind));
                } else if dir.is_dir() {
                    batch.push(FileEntry::from_path_with_kind(
                        dir.clone(),
                        EntryKind::Directory,
                    ));
                    let stats = scan_dir(&queue, &dir, &mut batch);
                    skipped_dirs += stats.skipped_dirs;
                    skipped_items += stats.skipped_items;
                }

                if batch.len() >= 4096 {
                    let ready = std::mem::take(&mut batch);
                    if sender
                        .send(ScanBatch {
                            entries: ready,
                            skipped_dirs,
                            skipped_items,
                        })
                        .is_err()
                    {
                        return;
                    }
                    skipped_dirs = 0;
                    skipped_items = 0;
                }
                queue.finish_one();
            }

            if !batch.is_empty() || skipped_dirs > 0 || skipped_items > 0 {
                let _ = sender.send(ScanBatch {
                    entries: batch,
                    skipped_dirs,
                    skipped_items,
                });
            }
        }));
    }

    drop(sender);

    let mut entries = Vec::new();
    let mut skipped_dirs = 0;
    let mut skipped_items = 0;
    for batch in receiver {
        entries.extend(batch.entries);
        skipped_dirs += batch.skipped_dirs;
        skipped_items += batch.skipped_items;
    }

    for worker in workers {
        worker
            .join()
            .map_err(|_| "index worker thread panicked while scanning")?;
    }

    sort_and_dedup(&mut entries);
    Ok(ScanReport {
        entries,
        skipped_dirs,
        skipped_items,
    })
}

#[derive(Debug, Default)]
struct ScanStats {
    skipped_dirs: usize,
    skipped_items: usize,
}

#[derive(Debug)]
struct ScanBatch {
    entries: Vec<FileEntry>,
    skipped_dirs: usize,
    skipped_items: usize,
}

fn scan_dir(queue: &WorkQueue, dir: &Path, batch: &mut Vec<FileEntry>) -> ScanStats {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return ScanStats {
            skipped_dirs: 1,
            skipped_items: 0,
        };
    };

    let mut stats = ScanStats::default();
    for item in read_dir {
        let Ok(item) = item else {
            stats.skipped_items += 1;
            continue;
        };
        let path = item.path();
        let Ok(file_type) = item.file_type() else {
            stats.skipped_items += 1;
            continue;
        };

        if file_type.is_dir() {
            if !is_probably_cycle_or_noise(&path) {
                batch.push(FileEntry::from_path_with_kind(
                    path.clone(),
                    EntryKind::Directory,
                ));
                queue.push(path);
            }
        } else if file_type.is_file() {
            let kind = file_kind(&path);
            batch.push(FileEntry::from_path_with_kind(path, kind));
        }
    }
    stats
}

fn file_kind(path: &Path) -> EntryKind {
    let extension = path
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase);

    match extension.as_deref() {
        Some("exe" | "lnk" | "appref-ms") => EntryKind::Application,
        _ => EntryKind::File,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn indexes_files_and_directories() {
        let dir = std::env::temp_dir().join(format!(
            "quickfind-scan-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let nested = dir.join("Project Reports");
        let file = nested.join("summary.txt");
        fs::create_dir_all(&nested).unwrap();
        fs::write(&file, "summary").unwrap();

        let entries = build_index(vec![dir.clone()], 2).unwrap();

        assert!(
            entries
                .iter()
                .any(|entry| entry.path == file.to_string_lossy())
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.path == nested.to_string_lossy()
                    && entry.kind == EntryKind::Directory)
        );

        let _ = fs::remove_file(file);
        let _ = fs::remove_dir(nested);
        let _ = fs::remove_dir(dir);
    }

    #[test]
    fn indexes_file_root() {
        let dir = std::env::temp_dir().join(format!(
            "quickfind-file-root-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("single.txt");
        fs::write(&file, "single").unwrap();

        let entries = build_index(vec![file.clone()], 1).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, file.to_string_lossy());
        assert_eq!(entries[0].kind, EntryKind::File);

        let _ = fs::remove_file(file);
        let _ = fs::remove_dir(dir);
    }

    #[test]
    fn classifies_windows_app_entries() {
        assert_eq!(
            file_kind(Path::new("C:\\Tools\\Quickfind.exe")),
            EntryKind::Application
        );
        assert_eq!(
            file_kind(Path::new("C:\\Users\\Public\\App.lnk")),
            EntryKind::Application
        );
        assert_eq!(file_kind(Path::new("C:\\Docs\\notes.txt")), EntryKind::File);
    }
}

fn is_probably_cycle_or_noise(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    matches!(
        name.to_ascii_lowercase().as_str(),
        "$recycle.bin" | "system volume information"
    )
}

#[derive(Debug)]
struct WorkQueue {
    inner: Mutex<QueueState>,
    ready: Condvar,
}

#[derive(Debug)]
struct QueueState {
    dirs: VecDeque<PathBuf>,
    active: usize,
    done: bool,
}

impl WorkQueue {
    fn new(roots: Vec<PathBuf>) -> Self {
        Self {
            inner: Mutex::new(QueueState {
                dirs: roots.into(),
                active: 0,
                done: false,
            }),
            ready: Condvar::new(),
        }
    }

    fn pop(&self) -> Option<PathBuf> {
        let mut state = self.inner.lock().expect("queue poisoned");
        loop {
            if let Some(dir) = state.dirs.pop_front() {
                state.active += 1;
                return Some(dir);
            }

            if state.done || state.active == 0 {
                state.done = true;
                self.ready.notify_all();
                return None;
            }

            state = self.ready.wait(state).expect("queue poisoned");
        }
    }

    fn push(&self, dir: PathBuf) {
        let mut state = self.inner.lock().expect("queue poisoned");
        if !state.done {
            state.dirs.push_back(dir);
            self.ready.notify_one();
        }
    }

    fn finish_one(&self) {
        let mut state = self.inner.lock().expect("queue poisoned");
        state.active -= 1;
        if state.active == 0 && state.dirs.is_empty() {
            state.done = true;
        }
        self.ready.notify_all();
    }
}
