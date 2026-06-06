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

  function kindLabel(kind: SearchItem["kind"]) {
    if (kind === "app") return "App";
    if (kind === "folder") return "Folder";
    return "File";
  }

  function iconLabel(kind: SearchItem["kind"]) {
    if (kind === "app") return "A";
    if (kind === "folder") return "F";
    return "D";
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
  <div class="result-main">
    <div class:app={item.kind === "app"} class:folder={item.kind === "folder"} class:file={item.kind === "file"} class="result-icon">
      {iconLabel(item.kind)}
    </div>
    <div class="result-copy">
      <div class="result-title">
        <div class="result-name">{item.name || fileName(item.path)}</div>
        <div class:app={item.kind === "app"} class:folder={item.kind === "folder"} class:file={item.kind === "file"} class="kind">
          {kindLabel(item.kind)}
        </div>
      </div>
      <div class="result-path">{item.path}</div>
    </div>
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
