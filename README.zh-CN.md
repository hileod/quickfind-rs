# Quickfind

语言: [English](README.md) | [简体中文](README.zh-CN.md) | [日本語](README.ja.md)

Quickfind 是一个面向 Windows 的本地文件搜索工具，灵感来自 Everything 和 uTools Find。核心索引/搜索逻辑使用 Rust 实现，桌面端使用 Tauri + Svelte。

当前版本使用并发递归扫描、紧凑的 `.qf` 本地热索引，以及 Turso `.meta.db` 元数据 sidecar。下一阶段目标是 Windows Service + MFT 首次建索引 + USN Journal 增量更新。

开源协议：MIT。

## 技术栈

- Rust stable，edition 2024：扫描器、搜索引擎、二进制索引、CLI、Tauri 后端。
- Tauri 2：桌面壳和前后端命令桥。
- Svelte 5 + TypeScript + Vite：桌面 UI。
- Turso：SQLite-compatible 本地元数据存储。
- 自研 `.qf` 二进制索引：用于低延迟加载和搜索热路径。
- Windows 路线：管理员重启、后续 NTFS/ReFS MFT + USN Journal。

## 亮点

- 桌面端支持建索引、搜索、类型过滤、结果分组和直接打开文件。
- 文件和文件夹都会建索引，并按 `file` / `folder` 分类展示。
- 支持 `*.pdf`、`budget *.pdf` 这类通配扩展名查询。
- 支持按类型、扩展名、盘符过滤。
- 搜索排序包含精确文件名、前缀、包含、路径包含、模糊匹配、文件夹加权和短路径 tie-break。
- 双击结果或点击 `Open` 可通过 Windows 默认程序打开文件/文件夹。
- 线程数支持自动适配，CLI 中 `--threads 0` 表示自动。
- 扫描遇到无权限目录不会导致整体失败，会跳过并在 UI 中显示统计。
- 支持通过 UAC 以管理员身份重新启动，方便扫描受保护目录。
- Tauri 打包配置支持 NSIS Windows 安装包。

## 项目结构

```text
src/
  scanner.rs             并发目录扫描与跳过统计
  search_engine/         查询解析、过滤、排序和模糊匹配
  storage.rs             `.qf` 二进制索引读写
  turso_storage.rs       Turso 元数据 sidecar
  windows_indexer/       MFT / USN / Windows Service 后续架构骨架

src-tauri/
  src/lib.rs             Tauri 命令和桌面端后端状态
  tauri.conf.json        Tauri 桌面与 NSIS 打包配置

frontend/
  src/App.svelte         桌面应用入口
  src/components/        索引、搜索、结果列表组件
  src/lib/tauri.ts       Tauri 命令封装
```

## 目标架构

```text
Rust Windows Service
  -> 枚举 NTFS/ReFS 卷
  -> 读取 MFT 建初始索引
  -> 监听 USN Journal 增量更新
  -> 更新混合索引存储
     -> .qf 热索引
     -> Turso metadata sidecar
     -> 可选 Tantivy 全文/字段索引
  -> Tauri GUI / egui GUI / CLI
```

## 桌面端使用

安装前端依赖：

```powershell
npm.cmd install --prefix frontend
```

开发模式运行：

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri dev
```

构建 debug 桌面程序：

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri build --debug
```

构建 Windows 安装包：

```powershell
Set-Location D:\quickfind-rs\src-tauri
cargo tauri build
```

安装包输出目录：

```text
D:\quickfind-rs\src-tauri\target\release\bundle\nsis\
```

默认桌面索引路径：

```text
<install-dir>\.quickfind\desktop.qf
<install-dir>\.quickfind\desktop.meta.db
```

如果安装在 `C:\Program Files` 等受保护目录，写入索引可能需要管理员权限。可以安装到用户可写目录，或在应用中点击 `Restart as administrator`。

## CLI 使用

构建：

```powershell
cargo build --release
```

创建或刷新索引：

```powershell
cargo run --release -- index C:\Users C:\Projects
```

搜索：

```powershell
cargo run --release -- search cargo.toml
cargo run --release -- search "report 2026" --limit 20
cargo run --release -- search "*.pdf" --limit 50
cargo run --release -- search "budget *.pdf" --limit 20
```

查看索引统计：

```powershell
cargo run --release -- stats
```

## 当前限制

- 当前递归扫描在超大磁盘上仍慢于 Everything 的 MFT 路线。
- 部分受保护 Windows 目录需要管理员权限，普通权限下会被跳过。
- 暂无驻留服务，需要手动刷新索引。
- USN Journal 增量更新还未真正接入。

## 下一步性能路线

- 使用 `FindFirstVolumeW` / `GetVolumeInformationW` 枚举卷。
- 通过 Windows volume handle 读取 MFT 建初始索引。
- 接入 USN Journal 监听创建、删除、重命名、修改事件。
- 用 file reference number + parent reference 解析完整路径。
- 增加常驻 Windows Service 保持索引热更新。
- 可选引入 Tantivy 支持内容搜索和更丰富字段索引。
