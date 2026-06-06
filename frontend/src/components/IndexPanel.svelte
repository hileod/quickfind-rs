<script lang="ts">
  import type { Defaults, StatsSummary } from "../lib/types";

  type Props = {
    root: string;
    index: string;
    threads: string;
    defaults: Defaults | null;
    status: string;
    statusKind: "" | "ok" | "error";
    stats: StatsSummary | null;
    busy: boolean;
    onBuild: () => void;
    onStats: () => void;
    onAdmin: () => void;
    onRootChange: (value: string) => void;
    onIndexChange: (value: string) => void;
    onThreadsChange: (value: string) => void;
  };

  let {
    root,
    index,
    threads,
    defaults,
    status,
    statusKind,
    stats,
    busy,
    onBuild,
    onStats,
    onAdmin,
    onRootChange,
    onIndexChange,
    onThreadsChange,
  }: Props = $props();
</script>

<aside class="sidebar">
  <header class="brand">
    <div class="mark">QF</div>
    <div>
      <h1>Quickfind</h1>
      <p>Local file index</p>
    </div>
  </header>

  <section class="panel">
    <h2>Index</h2>
    <label>
      Root
      <input spellcheck="false" value={root} oninput={(event) => onRootChange(event.currentTarget.value)} />
    </label>
    <label>
      Index file
      <input spellcheck="false" value={index} oninput={(event) => onIndexChange(event.currentTarget.value)} />
    </label>
    <label>
      Threads
      <input
        type="number"
        min="0"
        max="32"
        placeholder={defaults ? `Auto (${defaults.threads})` : "Auto"}
        value={threads}
        oninput={(event) => onThreadsChange(event.currentTarget.value)}
      />
    </label>
    <button class="primary" disabled={busy} onclick={onBuild}>Build index</button>
    <button disabled={busy} onclick={onAdmin}>Run as administrator</button>
    <div class:ok={statusKind === "ok"} class:error={statusKind === "error"} class="status">
      {status}
    </div>
  </section>

  <section class="panel compact">
    <h2>Stats</h2>
    <button disabled={busy} onclick={onStats}>Refresh stats</button>
    {#if stats}
      <dl>
        <dt>Files</dt>
        <dd>{stats.files.toLocaleString()}</dd>
        <dt>Path bytes</dt>
        <dd>{stats.pathBytes.toLocaleString()}</dd>
        <dt>Loaded</dt>
        <dd>{stats.loadMs.toFixed(2)} ms</dd>
        <dt>Index</dt>
        <dd>{stats.index}</dd>
        {#if stats.metadata}
          <dt>Metadata</dt>
          <dd>{stats.metadata}</dd>
        {/if}
      </dl>
    {/if}
  </section>
</aside>
