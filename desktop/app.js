const invoke = window.__TAURI__.core.invoke;

const rootInput = document.querySelector("#rootInput");
const indexInput = document.querySelector("#indexInput");
const threadsInput = document.querySelector("#threadsInput");
const indexButton = document.querySelector("#indexButton");
const indexStatus = document.querySelector("#indexStatus");
const statsButton = document.querySelector("#statsButton");
const statsList = document.querySelector("#statsList");
const queryInput = document.querySelector("#queryInput");
const limitInput = document.querySelector("#limitInput");
const kindFilter = document.querySelector("#kindFilter");
const extensionFilter = document.querySelector("#extensionFilter");
const driveFilter = document.querySelector("#driveFilter");
const searchButton = document.querySelector("#searchButton");
const results = document.querySelector("#results");
const resultCount = document.querySelector("#resultCount");
const timing = document.querySelector("#timing");
let searchTimer = null;
let searchSeq = 0;

function setIndexBusy(isBusy) {
  indexButton.disabled = isBusy;
  statsButton.disabled = isBusy;
}

function setSearchBusy(isBusy) {
  searchButton.disabled = isBusy;
}

function showStatus(message, className = "") {
  indexStatus.className = `status ${className}`.trim();
  indexStatus.textContent = message;
}

function formatMs(value) {
  return `${value.toFixed(2)} ms`;
}

function fileName(path) {
  const normalized = path.replaceAll("\\", "/");
  return normalized.slice(normalized.lastIndexOf("/") + 1) || path;
}

function renderStats(stats) {
  statsList.innerHTML = "";
  const rows = [
    ["Files", stats.files.toLocaleString()],
    ["Path bytes", stats.pathBytes.toLocaleString()],
    ["Loaded", formatMs(stats.loadMs)],
    ["Index", stats.index],
  ];
  if (stats.metadata) {
    rows.push(["Metadata", stats.metadata]);
  }

  for (const [label, value] of rows) {
    const dt = document.createElement("dt");
    const dd = document.createElement("dd");
    dt.textContent = label;
    dd.textContent = value;
    statsList.append(dt, dd);
  }
}

function renderResults(payload) {
  results.innerHTML = "";
  resultCount.textContent = `${payload.matches.length.toLocaleString()} results`;
  timing.textContent = `load ${formatMs(payload.loadMs)} / search ${formatMs(payload.searchMs)}`;

  if (payload.matches.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.textContent = "No matching files";
    results.append(empty);
    return;
  }

  const groups = [
    ["Folders", payload.matches.filter((item) => item.kind === "folder")],
    ["Files", payload.matches.filter((item) => item.kind !== "folder")],
  ];

  for (const [label, items] of groups) {
    if (items.length === 0) {
      continue;
    }

    const header = document.createElement("div");
    header.className = "result-group";
    header.textContent = `${label} (${items.length})`;
    results.append(header);

    for (const item of items) {
      results.append(renderResultRow(item));
    }
  }
}

function renderResultRow(item) {
  const row = document.createElement("div");
  row.className = "result-row";

  const main = document.createElement("div");
  const name = document.createElement("div");
  const path = document.createElement("div");
  name.className = "result-name";
  path.className = "result-path";
  name.textContent = item.name || fileName(item.path);
  path.textContent = item.path;

  const meta = document.createElement("div");
  meta.className = `kind ${item.kind === "folder" ? "folder" : "file"}`;
  meta.textContent = item.kind === "folder" ? "Folder" : "File";

  const title = document.createElement("div");
  title.className = "result-title";
  title.append(name, meta);
  main.append(title, path);

  const score = document.createElement("div");
  score.className = "score";
  score.textContent = item.score;

  row.append(main, score);
  return row;
}

async function loadDefaults() {
  const defaults = await invoke("get_defaults");
  rootInput.value = defaults.root;
  indexInput.value = defaults.index;
  threadsInput.value = "";
  threadsInput.placeholder = `Auto (${defaults.threads})`;
  showStatus("Ready");
}

async function buildIndex() {
  const root = rootInput.value.trim();
  const output = indexInput.value.trim();
  const threads = Number.parseInt(threadsInput.value, 10) || 0;

  if (!root || !output) {
    showStatus("Root and index file are required.", "error");
    return;
  }

  setIndexBusy(true);
  showStatus("Indexing...");
  try {
    const summary = await invoke("rebuild_index", {
      roots: [root],
      output,
      threads,
    });
    showStatus(
      `Indexed ${summary.files.toLocaleString()} files in ${formatMs(summary.elapsedMs)}. Metadata is writing in background: ${summary.metadata}`,
      "ok",
    );
    await refreshStats();
  } catch (error) {
    showStatus(String(error), "error");
  } finally {
    setIndexBusy(false);
  }
}

async function refreshStats() {
  const index = indexInput.value.trim();
  if (!index) {
    showStatus("Index file is required.", "error");
    return;
  }

  setIndexBusy(true);
  try {
    const stats = await invoke("index_stats", { index });
    renderStats(stats);
  } catch (error) {
    showStatus(String(error), "error");
  } finally {
    setIndexBusy(false);
  }
}

async function runSearch() {
  const seq = ++searchSeq;
  const query = queryInput.value.trim();
  const index = indexInput.value.trim();
  const limit = Number.parseInt(limitInput.value, 10) || 50;
  const kind = kindFilter.value;
  const extension = extensionFilter.value.trim();
  const drive = driveFilter.value.trim();
  const hasFilters = Boolean(kind || extension || drive);

  if (!query && !hasFilters) {
    resultCount.textContent = "Enter a query or filter";
    timing.textContent = "";
    results.innerHTML = "";
    return;
  }

  setSearchBusy(true);
  try {
    const payload = await invoke("search_index", {
      query,
      index,
      limit,
      kind,
      extension,
      drive,
    });
    if (seq !== searchSeq) {
      return;
    }
    renderResults(payload);
  } catch (error) {
    if (seq !== searchSeq) {
      return;
    }
    resultCount.textContent = "Search failed";
    timing.textContent = "";
    results.innerHTML = `<div class="empty">${String(error)}</div>`;
  } finally {
    if (seq === searchSeq) {
      setSearchBusy(false);
    }
  }
}

function scheduleSearch() {
  window.clearTimeout(searchTimer);
  searchTimer = window.setTimeout(runSearch, 200);
}

indexButton.addEventListener("click", buildIndex);
statsButton.addEventListener("click", refreshStats);
searchButton.addEventListener("click", runSearch);
queryInput.addEventListener("input", scheduleSearch);
limitInput.addEventListener("input", scheduleSearch);
kindFilter.addEventListener("change", scheduleSearch);
extensionFilter.addEventListener("input", scheduleSearch);
driveFilter.addEventListener("input", scheduleSearch);
queryInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    window.clearTimeout(searchTimer);
    runSearch();
  }
});

loadDefaults().catch((error) => showStatus(String(error), "error"));
