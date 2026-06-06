<script lang="ts">
  import IndexPanel from "./components/IndexPanel.svelte";
  import ResultList from "./components/ResultList.svelte";
  import SearchPanel from "./components/SearchPanel.svelte";
  import { getDefaults, indexStats, openPath, rebuildIndex, restartAsAdmin, scanRoots, searchIndex } from "./lib/tauri";
  import type { Defaults, EntryKind, SearchFilters, SearchPayload, StatsSummary } from "./lib/types";
  import "./app.css";

  let defaults = $state<Defaults | null>(null);
  let root = $state("");
  let index = $state("");
  let threads = $state("");
  let status = $state("Ready");
  let statusKind = $state<"" | "ok" | "error">("");
  let stats = $state<StatsSummary | null>(null);
  let indexBusy = $state(false);

  let query = $state("");
  let limit = $state(50);
  let kind = $state<EntryKind>("");
  let extension = $state("");
  let drive = $state("");
  let searchBusy = $state(false);
  let payload = $state<SearchPayload | null>(null);
  let searchError = $state("");
  let searched = $state(false);
  let searchTimer: number | undefined;
  let searchSeq = 0;

  let resultCount = $derived(payload ? `${payload.matches.length.toLocaleString()} results` : "No search yet");
  let timing = $derived(payload ? `load ${formatMs(payload.loadMs)} / search ${formatMs(payload.searchMs)}` : "");

  function formatMs(value: number) {
    return `${value.toFixed(2)} ms`;
  }

  function filters(): SearchFilters {
    return { kind, extension: extension.trim(), drive: drive.trim() };
  }

  function hasSearchInput() {
    const filter = filters();
    return Boolean(query.trim() || filter.kind || filter.extension || filter.drive);
  }

  function parseRoots(value: string) {
    return value
      .split(/[\n;,]+/)
      .map((item) => item.trim())
      .filter(Boolean);
  }

  function scheduleSearch() {
    window.clearTimeout(searchTimer);
    searchTimer = window.setTimeout(runSearch, 200);
  }

  async function buildIndex() {
    const output = index.trim();
    const scanRootList = parseRoots(root);
    const threadCount = Number.parseInt(threads, 10) || 0;

    if (!scanRootList.length || !output) {
      status = "Roots and index file are required.";
      statusKind = "error";
      return;
    }

    indexBusy = true;
    status = "Indexing...";
    statusKind = "";
    try {
      const summary = await rebuildIndex({
        roots: scanRootList,
        output,
        threads: threadCount,
      });
      const skipped = summary.skippedDirs || summary.skippedItems
        ? ` Skipped ${summary.skippedDirs.toLocaleString()} dirs / ${summary.skippedItems.toLocaleString()} items without access.`
        : "";
      status = `Indexed ${summary.files.toLocaleString()} files in ${formatMs(summary.elapsedMs)}.${skipped} Metadata is writing in background: ${summary.metadata}`;
      statusKind = "ok";
      await refreshStats();
    } catch (error) {
      status = String(error);
      statusKind = "error";
    } finally {
      indexBusy = false;
    }
  }

  async function refreshStats() {
    const currentIndex = index.trim();
    if (!currentIndex) {
      status = "Index file is required.";
      statusKind = "error";
      return;
    }

    indexBusy = true;
    try {
      stats = await indexStats(currentIndex);
    } catch (error) {
      status = String(error);
      statusKind = "error";
    } finally {
      indexBusy = false;
    }
  }

  async function runSearch() {
    const seq = ++searchSeq;
    if (!hasSearchInput()) {
      searched = false;
      payload = null;
      searchError = "";
      return;
    }

    searchBusy = true;
    try {
      const nextPayload = await searchIndex({
        query: query.trim(),
        index: index.trim(),
        limit,
        filters: filters(),
      });
      if (seq !== searchSeq) return;
      payload = nextPayload;
      searched = true;
      searchError = "";
    } catch (error) {
      if (seq !== searchSeq) return;
      payload = null;
      searched = true;
      searchError = String(error);
    } finally {
      if (seq === searchSeq) searchBusy = false;
    }
  }

  async function openResult(path: string) {
    try {
      await openPath(path);
      searchError = "";
      status = `Opened ${path}`;
      statusKind = "ok";
    } catch (error) {
      searchError = String(error);
      status = String(error);
      statusKind = "error";
    }
  }

  async function relaunchAdmin() {
    try {
      await restartAsAdmin();
      status = "UAC prompt opened. Close this window after the elevated Quickfind starts.";
      statusKind = "ok";
    } catch (error) {
      status = String(error);
      statusKind = "error";
    }
  }

  async function useAllDrives() {
    try {
      const roots = await scanRoots();
      if (!roots.length) {
        status = "No scan drives were found.";
        statusKind = "error";
        return;
      }

      root = roots.join("; ");
      status = `Ready to scan ${roots.join(", ")}`;
      statusKind = "ok";
    } catch (error) {
      status = String(error);
      statusKind = "error";
    }
  }

  getDefaults()
    .then((value) => {
      defaults = value;
      root = value.root;
      index = value.index;
    })
    .catch((error) => {
      status = String(error);
      statusKind = "error";
    });
</script>

<main class="app-shell">
  <IndexPanel
    {root}
    {index}
    {threads}
    {defaults}
    {status}
    {statusKind}
    {stats}
    busy={indexBusy}
    onBuild={buildIndex}
    onStats={refreshStats}
    onAdmin={relaunchAdmin}
    onAllDrives={useAllDrives}
    onRootChange={(value) => (root = value)}
    onIndexChange={(value) => (index = value)}
    onThreadsChange={(value) => (threads = value)}
  />

  <section class="workspace">
    <header class="workspace-head">
      <div>
        <h2>Search</h2>
        <p>Filename, path, wildcard extension, and typed filters</p>
      </div>
      <div class="quick-hints">
        <span>*.pdf</span>
        <span>budget *.xlsx</span>
        <span>folder names</span>
      </div>
    </header>

    <SearchPanel
      {query}
      {limit}
      {kind}
      {extension}
      {drive}
      busy={searchBusy}
      onSearch={runSearch}
      onQueryChange={(value) => {
        query = value;
        scheduleSearch();
      }}
      onLimitChange={(value) => {
        limit = value;
        scheduleSearch();
      }}
      onKindChange={(value) => {
        kind = value;
        scheduleSearch();
      }}
      onExtensionChange={(value) => {
        extension = value;
        scheduleSearch();
      }}
      onDriveChange={(value) => {
        drive = value;
        scheduleSearch();
      }}
    />

    <div class="results-meta">
      <span>{payload || searched ? resultCount : "No search yet"}</span>
      <span>{timing}</span>
    </div>

    <div class="results">
      <ResultList {payload} error={searchError} {searched} onOpen={openResult} />
    </div>
  </section>
</main>
