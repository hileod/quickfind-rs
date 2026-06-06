import { invoke } from "@tauri-apps/api/core";
import type { Defaults, IndexSummary, SearchFilters, SearchPayload, StatsSummary } from "./types";

export function getDefaults() {
  return invoke<Defaults>("get_defaults");
}

export function scanRoots() {
  return invoke<string[]>("scan_roots");
}

export function rebuildIndex(args: { roots: string[]; output: string; threads: number }) {
  return invoke<IndexSummary>("rebuild_index", args);
}

export function indexStats(index: string) {
  return invoke<StatsSummary>("index_stats", { index });
}

export function searchIndex(args: {
  query: string;
  index: string;
  limit: number;
  filters: SearchFilters;
}) {
  return invoke<SearchPayload>("search_index", {
    query: args.query,
    index: args.index,
    limit: args.limit,
    kind: args.filters.kind,
    extension: args.filters.extension,
    drive: args.filters.drive,
  });
}

export function openPath(path: string) {
  return invoke<void>("open_path", { path });
}

export function restartAsAdmin() {
  return invoke<void>("restart_as_admin");
}
