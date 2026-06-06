use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::time::SystemTime;

use quickfind::cli;
use quickfind::index::{EntryKind, FileEntry};
use quickfind::scanner::build_index_with_report;
use quickfind::search_engine;
use quickfind::storage;
use quickfind::turso_storage;
use serde::Serialize;
use tauri::State;

type AppResult<T> = Result<T, String>;

#[derive(Default)]
struct AppState {
    cache: Mutex<Option<CachedIndex>>,
}

#[derive(Clone)]
struct CachedIndex {
    path: PathBuf,
    modified: Option<SystemTime>,
    entries: Arc<Vec<FileEntry>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Defaults {
    root: String,
    index: String,
    threads: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IndexSummary {
    files: usize,
    index: String,
    metadata: String,
    metadata_status: String,
    skipped_dirs: usize,
    skipped_items: usize,
    elapsed_ms: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatsSummary {
    index: String,
    metadata: Option<String>,
    files: usize,
    path_bytes: usize,
    load_ms: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchPayload {
    matches: Vec<SearchItem>,
    load_ms: f64,
    search_ms: f64,
}

#[derive(Debug, Serialize)]
struct SearchItem {
    path: String,
    name: String,
    kind: String,
    score: i64,
}

#[tauri::command]
fn get_defaults() -> Defaults {
    let root = std::env::current_dir().unwrap_or_else(|_| cli::default_root());
    let index = default_data_dir().join("desktop.qf");

    Defaults {
        root: root.display().to_string(),
        index: index.display().to_string(),
        threads: cli::default_thread_count(),
    }
}

#[tauri::command]
async fn rebuild_index(
    state: State<'_, AppState>,
    roots: Vec<String>,
    output: String,
    threads: usize,
) -> AppResult<IndexSummary> {
    let summary = tauri::async_runtime::spawn_blocking(move || {
        let roots = roots
            .into_iter()
            .filter(|root| !root.trim().is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        if roots.is_empty() {
            return Err("at least one root directory is required".to_string());
        }

        let output = PathBuf::from(output);
        let started = Instant::now();
        let thread_count = if threads == 0 {
            cli::default_thread_count()
        } else {
            threads.clamp(1, 32)
        };
        let report = build_index_with_report(roots, thread_count).map_err(to_message)?;
        storage::write_index(&output, &report.entries).map_err(to_message)?;

        let metadata = turso_storage::metadata_path_for(&output);
        let modified = index_modified(&output);
        let entries = Arc::new(report.entries);
        let cached = CachedIndex {
            path: output.clone(),
            modified,
            entries,
        };

        Ok((
            IndexSummary {
                files: cached.entries.len(),
                index: output.display().to_string(),
                metadata: metadata.display().to_string(),
                metadata_status: String::from("queued"),
                skipped_dirs: report.skipped_dirs,
                skipped_items: report.skipped_items,
                elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
            },
            metadata,
            cached,
        ))
    })
    .await
    .map_err(to_message)??;

    let (summary, metadata, cached) = summary;
    replace_cache(&state, cached)?;
    write_metadata_in_background(metadata, Arc::clone(&current_cache_required(&state)?));
    Ok(summary)
}

#[tauri::command]
async fn index_stats(state: State<'_, AppState>, index: String) -> AppResult<StatsSummary> {
    let index = PathBuf::from(index);
    let started = Instant::now();
    let entries = load_cached_entries(&state, &index).await?;
    let metadata = turso_storage::metadata_path_for(&index);

    Ok(StatsSummary {
        index: index.display().to_string(),
        metadata: metadata.exists().then(|| metadata.display().to_string()),
        files: entries.len(),
        path_bytes: entries.iter().map(|entry| entry.path.len()).sum(),
        load_ms: started.elapsed().as_secs_f64() * 1000.0,
    })
}

#[tauri::command]
async fn search_index(
    state: State<'_, AppState>,
    query: String,
    index: String,
    limit: usize,
    kind: Option<String>,
    extension: Option<String>,
    drive: Option<String>,
) -> AppResult<SearchPayload> {
    let started = Instant::now();
    let entries = load_cached_entries(&state, &PathBuf::from(index)).await?;
    let loaded_at = started.elapsed();
    let searched = Instant::now();
    let filters = search_engine::SearchFilters {
        kind: parse_kind_filter(kind.as_deref()),
        extension: normalize_extension(extension.as_deref()),
        drive: normalize_drive(drive.as_deref()),
    };
    let matches = search_engine::search_with_filters(&entries, &query, limit.max(1), &filters)
        .into_iter()
        .map(|hit| SearchItem {
            path: hit.entry.path.clone(),
            name: hit.entry.name().to_string(),
            kind: hit.entry.kind.as_str().to_string(),
            score: hit.score,
        })
        .collect();

    Ok(SearchPayload {
        matches,
        load_ms: loaded_at.as_secs_f64() * 1000.0,
        search_ms: searched.elapsed().as_secs_f64() * 1000.0,
    })
}

#[tauri::command]
async fn open_path(path: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(path.trim());
        if path.as_os_str().is_empty() {
            return Err(String::from("path is required"));
        }

        if !path.exists() {
            return Err(format!("path does not exist: {}", path.display()));
        }

        Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(to_message)
    })
    .await
    .map_err(to_message)?
}

#[tauri::command]
async fn restart_as_admin() -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let exe = std::env::current_exe().map_err(to_message)?;
        let command = format!("Start-Process -FilePath '{}' -Verb RunAs", escape_ps_path(&exe));
        Command::new("powershell.exe")
            .args(["-NoProfile", "-Command", &command])
            .spawn()
            .map(|_| ())
            .map_err(to_message)
    })
    .await
    .map_err(to_message)?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_defaults,
            rebuild_index,
            index_stats,
            search_index,
            open_path,
            restart_as_admin
        ])
        .run(tauri::generate_context!())
        .expect("failed to run quickfind desktop app");
}

fn to_message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

async fn load_cached_entries(
    state: &State<'_, AppState>,
    path: &PathBuf,
) -> AppResult<Arc<Vec<FileEntry>>> {
    let modified = index_modified(path);
    if let Some(entries) = current_cache(state, path, modified)? {
        return Ok(entries);
    }

    let path_for_load = path.clone();
    let entries = tauri::async_runtime::spawn_blocking(move || {
        storage::read_index(&path_for_load)
            .map(Arc::new)
            .map_err(to_message)
            .map(|entries| {
                let modified = index_modified(&path_for_load);
                (
                    CachedIndex {
                        path: path_for_load,
                        modified,
                        entries: Arc::clone(&entries),
                    },
                    entries,
                )
            })
    })
    .await
    .map_err(to_message)??;

    let (cached, entries) = entries;
    replace_cache(state, cached)?;
    Ok(entries)
}

fn current_cache(
    state: &State<'_, AppState>,
    path: &PathBuf,
    modified: Option<SystemTime>,
) -> AppResult<Option<Arc<Vec<FileEntry>>>> {
    let cache = state.cache.lock().map_err(to_message)?;
    Ok(cache.as_ref().and_then(|cached| {
        (cached.path == *path && cached.modified == modified).then(|| Arc::clone(&cached.entries))
    }))
}

fn replace_cache(state: &State<'_, AppState>, cached: CachedIndex) -> AppResult<()> {
    let mut cache = state.cache.lock().map_err(to_message)?;
    *cache = Some(cached);
    Ok(())
}

fn current_cache_required(state: &State<'_, AppState>) -> AppResult<Arc<Vec<FileEntry>>> {
    let cache = state.cache.lock().map_err(to_message)?;
    cache
        .as_ref()
        .map(|cached| Arc::clone(&cached.entries))
        .ok_or_else(|| String::from("index cache was not initialized"))
}

fn write_metadata_in_background(path: PathBuf, entries: Arc<Vec<FileEntry>>) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = turso_storage::write_metadata(&path, &entries) {
            eprintln!("metadata write failed for {}: {error}", path.display());
        }
    });
}

fn index_modified(path: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

fn parse_kind_filter(value: Option<&str>) -> Option<EntryKind> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some("file") => Some(EntryKind::File),
        Some("folder") | Some("directory") => Some(EntryKind::Directory),
        _ => None,
    }
}

fn normalize_extension(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .map(|value| value.trim_start_matches('.').to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}

fn normalize_drive(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .map(|value| value.trim_end_matches('\\').trim_end_matches('/'))
        .map(|value| {
            if value.len() == 1 {
                format!("{}:", value.to_ascii_uppercase())
            } else {
                value.to_ascii_uppercase()
            }
        })
        .filter(|value| !value.is_empty())
}

fn escape_ps_path(path: &std::path::Path) -> String {
    path.display().to_string().replace('\'', "''")
}

fn default_data_dir() -> PathBuf {
    install_dir()
        .unwrap_or_else(|| {
            std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| cli::default_root()))
                .join("Quickfind")
        })
        .join(".quickfind")
}

fn install_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_kind_filter() {
        assert_eq!(parse_kind_filter(Some("file")), Some(EntryKind::File));
        assert_eq!(parse_kind_filter(Some("folder")), Some(EntryKind::Directory));
        assert_eq!(parse_kind_filter(Some("")), None);
    }

    #[test]
    fn normalizes_extension_filter() {
        assert_eq!(normalize_extension(Some(".PDF")).as_deref(), Some("pdf"));
        assert_eq!(normalize_extension(Some("")).as_deref(), None);
    }

    #[test]
    fn normalizes_drive_filter() {
        assert_eq!(normalize_drive(Some("c")).as_deref(), Some("C:"));
        assert_eq!(normalize_drive(Some("d:\\")).as_deref(), Some("D:"));
    }
}
