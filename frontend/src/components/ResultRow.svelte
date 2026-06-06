<script lang="ts">
  import type { SearchItem } from "../lib/types";

  type Props = {
    item: SearchItem;
    onOpen: (path: string) => void;
  };

  let { item, onOpen }: Props = $props();

  function fileName(path: string) {
    const normalized = path.replaceAll("\\", "/");
    return normalized.slice(normalized.lastIndexOf("/") + 1) || path;
  }
</script>

<div
  class="result-row"
  role="button"
  tabindex="0"
  ondblclick={() => onOpen(item.path)}
  onkeydown={(event) => {
    if (event.key === "Enter") onOpen(item.path);
  }}
  title="Double-click to open"
>
  <div>
    <div class="result-title">
      <div class="result-name">{item.name || fileName(item.path)}</div>
      <div class:folder={item.kind === "folder"} class:file={item.kind !== "folder"} class="kind">
        {item.kind === "folder" ? "Folder" : "File"}
      </div>
    </div>
    <div class="result-path">{item.path}</div>
  </div>
  <div class="result-actions">
    <button
      class="open-button"
      type="button"
      onclick={(event) => {
        event.stopPropagation();
        onOpen(item.path);
      }}>Open</button
    >
    <div class="score">{item.score}</div>
  </div>
</div>
