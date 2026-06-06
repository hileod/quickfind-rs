# Quickfind

Languages: [English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

Quickfind is a Windows-focused local file search tool inspired by Everything and uTools Find.
It is built with Rust for the indexing/search core and Tauri + Svelte for the desktop UI.

License: MIT.

The current implementation uses fast recursive scanning, a compact local `.qf` hot index, and a Turso `.meta.db` metadata sidecar. The next architecture step is a Windows Service backed by MFT initial indexing and USN Journal incremental updates.

## Tech Stack

- Rust 2024: scanner, search engine, binary index, CLI, and Tauri backend.
- Tauri 2: desktop shell and secure frontend/backend command bridge.
- Svelte 5 + TypeScript + Vite: desktop UI.
- Turso SQLite-compatible storage: durable metadata sidecar.
- Custom `.qf` binary index: compact hot-path search index for low-latency loading.
- Windows target path: administrator relaunch, future NTFS/ReFS MFT + USN Journal support.

## Highlights

- Desktop app with index controls, search box, typed filters, result grouping, and direct open actions.
- Fast name/path search with explicit ranking rules.
- Supports folder and file classification, similar to uTools Find result grouping.
- Supports wildcard extension queries such as `*.pdf` and combined queries such as `budget *.pdf`.
- Supports filters by type, extension, and drive.
- Double-click or `Open` button opens files/folders through Windows Explorer/default apps.
- Thread count can be automatic; CLI `--threads 0` means auto.
- Recursive scanner skips inaccessible directories/items instead of failing the whole scan.
- UI reports skipped permission-denied directories/items after indexing.
- Desktop app can run a new elevated instance through native UAC for broader scan access.
- Default desktop index path follows the installed executable directory: `<install-dir>\.quickfind\desktop.qf`.
- Metadata sidecar is written in the background so the UI returns quickly after indexing.

## Project Layout

```text
src/
  cli.rs                 CLI argument parsing and defaults
  index.rs               indexed file entry model and normalization
  scanner.rs             parallel directory traversal and skip reporting
  search.rs              legacy fuzzy search
  search_engine/
    document.rs          searchable document fields
    tokenizer.rs         query tokenization
    fuzzy_match.rs       fuzzy subsequence matching
    query.rs             query parsing and wildcard extension extraction
    ranking.rs           ranking rules
  windows_indexer/
    volume_scanner.rs    NTFS/ReFS volume enumeration scaffold
    mft_reader.rs        MFT initial snapshot scaffold
    usn_watcher.rs       USN Journal watcher scaffold
    path_resolver.rs     file-id + parent-id path resolution
    index_store.rs       hybrid index persistence facade
    service.rs           Windows Service scaffold
  storage.rs             `.qf` binary index read/write
  turso_storage.rs       Turso metadata sidecar read/write
  lib.rs                 command orchestration
  main.rs                process entry point

src-tauri/
  src/lib.rs             Tauri commands and desktop backend state
  tauri.conf.json        Tauri desktop config

frontend/
  src/App.svelte         desktop app composition
  src/components/        index/search/results UI
  src/lib/tauri.ts       Tauri command wrappers
```

## Target Architecture

```text
Rust Windows Service
  -> enumerate all NTFS/ReFS volumes
  -> read MFT for initial index
  -> watch USN Journal for incremental updates
  -> update hybrid index store
     -> .qf hot in-memory/search index
     -> Turso metadata sidecar
     -> optional Tantivy full-text/field index
  -> Tauri GUI / egui GUI / CLI
```

The current working implementation still uses recursive scanning. The `windows_indexer` module is the scaffold for the next phase: volume enumeration, MFT snapshot loading, USN Journal watching, file-id path resolution, and a resident Windows Service host.

## Search Model

Quickfind follows a local, in-process search model similar to the useful parts of Meilisearch, without running a search server.

```text
Document fields:
  name
  path
  kind: file | folder
  extension
  drive

Filters:
  kind
  extension
  drive

Ranking rules:
  exact filename
  filename prefix
  filename contains
  path contains
  fuzzy subsequence
  folder bonus
  shorter path tie-break
```

Wildcard extension queries such as `*.pdf` are converted into extension filters, so they can list matching files even when no filename term is provided.

## Desktop Usage

Install frontend dependencies:

```powershell
npm.cmd install --prefix frontend
```

Run the desktop app in development:

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri dev
```

Build a debug desktop executable:

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri build --debug
```

Run the built debug executable:

```powershell
Set-Location D:\quickfind-rs\src-tauri
.\target\debug\quickfind-desktop.exe
```

Build a Windows installer package:

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri build
```

The installer is generated under:

```text
D:\quickfind-rs\src-tauri\target\release\bundle\nsis\
```

The NSIS installer uses current-user installation by default to avoid `Program Files` write permission issues. The desktop app stores its default index beside the installed executable:

```text
<install-dir>\.quickfind\desktop.qf
<install-dir>\.quickfind\desktop.meta.db
```

If the app is installed under a protected directory such as `C:\Program Files`, writing the index beside the executable requires elevated permissions. Install to a user-writable path, or use `Run as administrator` in the app before indexing protected locations.

Runtime logs are written beside the installed executable:

```text
<install-dir>\logs\quickfind.log
```

For full-drive scanning such as `C:\`, use `Run as administrator` in the app if normal scanning reports many skipped directories. Long term, the planned Windows Service will handle privileged indexing more cleanly than an elevated GUI process.

## CLI Usage

Build:

```powershell
cargo build --release
```

Create or refresh an index:

```powershell
cargo run --release -- index C:\Users C:\Projects
```

Create an index with an explicit output path:

```powershell
cargo run --release -- index D:\quickfind-rs --output .\tmp-e2e\demo.qf --threads 4
```

The command above writes:

```text
.\tmp-e2e\demo.qf
.\tmp-e2e\demo.meta.db
```

Search:

```powershell
cargo run --release -- search cargo.toml
cargo run --release -- search "report 2026" --limit 20
cargo run --release -- search "*.pdf" --limit 50
cargo run --release -- search "budget *.pdf" --limit 20
cargo run --release -- search quickfind --index .\tmp-e2e\demo.qf --limit 20
```

Show index stats:

```powershell
cargo run --release -- stats
```

Default CLI index path:

```text
%LOCALAPPDATA%\quickfind\index.qf
%LOCALAPPDATA%\quickfind\index.meta.db
```

Default desktop index path:

```text
<install-dir>\.quickfind\desktop.qf
<install-dir>\.quickfind\desktop.meta.db
```

## Current Limitations

- Recursive scanning is slower than Everything-style MFT indexing on very large volumes.
- Some protected Windows directories require administrator privileges and may still be skipped.
- No resident service yet, so the index is refreshed manually.
- No real USN Journal incremental update loop yet.

## Next Performance Steps

- Real volume enumeration with `FindFirstVolumeW` / `GetVolumeInformationW`.
- MFT initial indexing through Windows volume handles.
- USN Journal watcher for create/delete/rename/modify events.
- File reference number plus parent reference path resolution.
- Resident Windows Service to keep the index hot.
- Memory-mapped `.qf` loading or compact in-memory trie/ngram candidate index.
- Optional Tantivy index for content or richer field search.
