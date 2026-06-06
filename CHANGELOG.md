# Changelog

## v0.1.3 - 2026-06-06

### Added

- Add desktop all-drive scan defaults and a `Use all drives` action.
- Classify Windows application entries from `.exe`, `.lnk`, and `.appref-ms` files.
- Add application result grouping and an `Apps` type filter in the desktop UI.

### Changed

- Support multiple desktop scan roots separated by semicolons, commas, or newlines.
- Preserve application entry kinds in the compact `.qf` index format.

## v0.1.2 - 2026-06-06

### Changed

- Improve Windows administrator relaunch behavior.
- Improve desktop logging around elevation and Windows startup.
- Build a fresh NSIS installer from the latest `main` source.

## v0.1.1 - 2026-06-06

### Added

- Add MIT open-source license.
- Add multilingual README files for English, Simplified Chinese, and Japanese.
- Add NSIS installer packaging configuration for the Tauri desktop app.

### Changed

- Bump project version to `0.1.1`.
- Document the installer output path and default desktop index location.

### Highlights

- Rust 2024 search core with a compact `.qf` hot index.
- Tauri 2 + Svelte 5 desktop UI.
- Turso metadata sidecar.
- Wildcard extension search such as `*.pdf`.
- File/folder grouping, typed filters, double-click open, permission skip reporting, and administrator relaunch support.
