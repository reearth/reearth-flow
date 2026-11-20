export type FeatureRowsStatus = {
  url: string;
  rowCount: number;
  totalRows: number;
  viewCount: number;
  searchTerm: string;
  searching: boolean;
  done: boolean;
  truncated: boolean;
  skippedLines: number;
  columns: string[];
  geometryType: string | null;
  error: string | null;
};

export type FeatureRowRecord = {
  row: number;
  id: unknown;
  attributes: Record<string, unknown>;
};

export type WorkerRequest =
  | { type: "open"; url: string }
  | { type: "clear" }
  | { type: "search"; url: string; term: string }
  | { type: "rows"; id: number; url: string; start: number; count: number }
  | { type: "line"; id: number; url: string; row: number }
  | { type: "viewIndex"; id: number; url: string; row: number };

export type StatusMessage = Omit<FeatureRowsStatus, "columns"> & {
  columns?: string[];
};

export type WorkerResponse =
  | { type: "status"; status: StatusMessage }
  | { type: "rows"; id: number; searchTerm: string; rows: FeatureRowRecord[] }
  | { type: "line"; id: number; text: string | null }
  | { type: "viewIndex"; id: number; index: number };
