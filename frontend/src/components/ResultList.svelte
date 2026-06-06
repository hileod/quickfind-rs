<script lang="ts">
  import type { SearchPayload } from "../lib/types";
  import ResultRow from "./ResultRow.svelte";

  type Props = {
    payload: SearchPayload | null;
    error: string;
    searched: boolean;
    onOpen: (path: string) => void;
  };

  let { payload, error, searched, onOpen }: Props = $props();

  let folders = $derived(payload?.matches.filter((item) => item.kind === "folder") ?? []);
  let files = $derived(payload?.matches.filter((item) => item.kind !== "folder") ?? []);

</script>

{#if error}
  <div class="empty">{error}</div>
{:else if !searched}
  <div class="empty">Enter a query, wildcard, or filter to search the current index.</div>
{:else if payload && payload.matches.length === 0}
  <div class="empty">No matching files</div>
{:else if payload}
  {#if folders.length}
    <div class="result-group">Folders ({folders.length})</div>
    {#each folders as item}
      <ResultRow {item} {onOpen} />
    {/each}
  {/if}

  {#if files.length}
    <div class="result-group">Files ({files.length})</div>
    {#each files as item}
      <ResultRow {item} {onOpen} />
    {/each}
  {/if}
{/if}
