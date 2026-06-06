export type EntryKind = "" | "file" | "folder";

export type Defaults = {
  root: string;
  index: string;
  threads: number;
};

export type IndexSummary = {
  files: number;
  index: string;
  metadata: string;
  metadataStatus: string;
  skippedDirs: number;
  skippedItems: number;
  elapsedMs: number;
};

export type StatsSummary = {
  index: string;
  metadata?: string;
  files: number;
  pathBytes: number;
  loadMs: number;
};

export type SearchItem = {
  path: string;
  name: string;
  kind: "file" | "folder";
  score: number;
};

export type SearchPayload = {
  matches: SearchItem[];
  loadMs: number;
  searchMs: number;
};

export type SearchFilters = {
  kind: EntryKind;
  extension: string;
  drive: string;
};
