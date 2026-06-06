<script lang="ts">
  import type { EntryKind } from "../lib/types";

  type Props = {
    query: string;
    limit: number;
    kind: EntryKind;
    extension: string;
    drive: string;
    busy: boolean;
    onSearch: () => void;
    onQueryChange: (value: string) => void;
    onLimitChange: (value: number) => void;
    onKindChange: (value: EntryKind) => void;
    onExtensionChange: (value: string) => void;
    onDriveChange: (value: string) => void;
  };

  let {
    query,
    limit,
    kind,
    extension,
    drive,
    busy,
    onSearch,
    onQueryChange,
    onLimitChange,
    onKindChange,
    onExtensionChange,
    onDriveChange,
  }: Props = $props();
</script>

<section class="search-panel">
  <div class="searchbar">
    <input
      id="queryInput"
      placeholder="Search files..."
      spellcheck="false"
      value={query}
      oninput={(event) => onQueryChange(event.currentTarget.value)}
      onkeydown={(event) => {
        if (event.key === "Enter") onSearch();
      }}
    />
    <input
      id="limitInput"
      type="number"
      min="1"
      max="500"
      value={limit}
      oninput={(event) => onLimitChange(Number.parseInt(event.currentTarget.value, 10) || 50)}
    />
    <button id="searchButton" disabled={busy} onclick={onSearch}>Search</button>
  </div>

  <div class="filters">
    <label>
      Type
      <select value={kind} onchange={(event) => onKindChange(event.currentTarget.value as EntryKind)}>
        <option value="">All</option>
        <option value="app">Apps</option>
        <option value="folder">Folders</option>
        <option value="file">Files</option>
      </select>
    </label>
    <label>
      Extension
      <input
        placeholder="pdf"
        spellcheck="false"
        value={extension}
        oninput={(event) => onExtensionChange(event.currentTarget.value)}
      />
    </label>
    <label>
      Drive
      <input
        placeholder="C:"
        spellcheck="false"
        value={drive}
        oninput={(event) => onDriveChange(event.currentTarget.value)}
      />
    </label>
  </div>
</section>
