# Quickfind

Languages: [English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

Quickfind は Windows 向けのローカルファイル検索ツールです。Everything と uTools Find に着想を得ており、インデックス作成と検索コアは Rust、デスクトップ UI は Tauri + Svelte で実装しています。

現在の実装は、高速な再帰スキャン、コンパクトな `.qf` ホットインデックス、Turso `.meta.db` メタデータ sidecar を使います。次の段階では Windows Service、MFT 初期インデックス、USN Journal による増分更新を目指します。

## 技術スタック

- Rust stable、edition 2024: スキャナ、検索エンジン、バイナリインデックス、CLI、Tauri バックエンド。
- Tauri 2: デスクトップシェルとフロントエンド/バックエンド間のコマンドブリッジ。
- Svelte 5 + TypeScript + Vite: デスクトップ UI。
- Turso: SQLite-compatible なローカルメタデータストレージ。
- 独自 `.qf` バイナリインデックス: 低レイテンシな読み込みと検索ホットパス。
- Windows 方針: 管理者としての再起動、将来的な NTFS/ReFS MFT + USN Journal 対応。

## 主な機能

- デスクトップ UI でインデックス作成、検索、型フィルタ、結果グループ化、ファイルを直接開く操作に対応。
- ファイルとフォルダをどちらもインデックスし、`file` / `folder` として分類表示。
- `*.pdf` や `budget *.pdf` のような拡張子ワイルドカード検索。
- 種類、拡張子、ドライブによるフィルタ。
- 検索ランキングは完全一致、接頭辞一致、ファイル名包含、パス包含、あいまい一致、フォルダ加点、短いパス優先を考慮。
- ダブルクリックまたは `Open` ボタンで、Windows の既定アプリ/Explorer からファイルやフォルダを開けます。
- スレッド数は自動調整可能。CLI の `--threads 0` は auto を意味します。
- アクセス権限のないディレクトリで全体を失敗させず、スキップして UI に件数を表示。
- UAC 経由で管理者として再起動できます。
- Tauri の NSIS Windows インストーラ設定を含みます。

## プロジェクト構成

```text
src/
  scanner.rs             並列ディレクトリスキャンとスキップ統計
  search_engine/         クエリ解析、フィルタ、ランキング、あいまい検索
  storage.rs             `.qf` バイナリインデックス読み書き
  turso_storage.rs       Turso メタデータ sidecar
  windows_indexer/       MFT / USN / Windows Service 向けの将来構成

src-tauri/
  src/lib.rs             Tauri コマンドとデスクトップバックエンド状態
  tauri.conf.json        Tauri デスクトップ/NSIS パッケージ設定

frontend/
  src/App.svelte         デスクトップアプリのエントリ
  src/components/        インデックス、検索、結果リスト UI
  src/lib/tauri.ts       Tauri コマンドラッパー
```

## 目標アーキテクチャ

```text
Rust Windows Service
  -> NTFS/ReFS ボリュームを列挙
  -> MFT を読んで初期インデックスを作成
  -> USN Journal で増分更新を監視
  -> ハイブリッドインデックスストアを更新
     -> .qf ホットインデックス
     -> Turso metadata sidecar
     -> optional Tantivy full-text/field index
  -> Tauri GUI / egui GUI / CLI
```

## デスクトップでの使い方

フロントエンド依存関係をインストール:

```powershell
npm.cmd install --prefix frontend
```

開発モードで起動:

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri dev
```

debug デスクトップ実行ファイルをビルド:

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri build --debug
```

Windows インストーラをビルド:

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri build
```

インストーラ出力先:

```text
D:\quickfind-rs\src-tauri\target\release\bundle\nsis\
```

既定のデスクトップインデックスパス:

```text
<install-dir>\.quickfind\desktop.qf
<install-dir>\.quickfind\desktop.meta.db
```

`C:\Program Files` など保護された場所にインストールした場合、インデックスの書き込みには管理者権限が必要になることがあります。ユーザーが書き込める場所にインストールするか、アプリ内の `Restart as administrator` を使ってください。

## CLI の使い方

ビルド:

```powershell
cargo build --release
```

インデックスを作成/更新:

```powershell
cargo run --release -- index C:\Users C:\Projects
```

検索:

```powershell
cargo run --release -- search cargo.toml
cargo run --release -- search "report 2026" --limit 20
cargo run --release -- search "*.pdf" --limit 50
cargo run --release -- search "budget *.pdf" --limit 20
```

インデックス統計:

```powershell
cargo run --release -- stats
```

## 現在の制限

- 現在の再帰スキャンは、大容量ボリュームでは Everything の MFT 方式より遅くなります。
- 一部の保護された Windows ディレクトリは管理者権限が必要で、通常権限ではスキップされます。
- 常駐サービスはまだありません。インデックスは手動で更新します。
- USN Journal による実際の増分更新は未実装です。

## 次の性能改善

- `FindFirstVolumeW` / `GetVolumeInformationW` によるボリューム列挙。
- Windows volume handle 経由で MFT を読み、初期インデックスを作成。
- USN Journal で作成、削除、リネーム、変更イベントを監視。
- file reference number + parent reference でフルパスを復元。
- 常駐 Windows Service でインデックスをホットに保つ。
- 必要に応じて Tantivy を導入し、内容検索や高度なフィールド検索に対応。
