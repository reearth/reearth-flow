import { ApiResponse } from "./api";

export type ProjectSnapshotMeta = {
  timestamp: string;
  version: number;
};

// A labelled, named snapshot in a project's version history (distinct from
// ProjectSnapshot, which is a raw update-vector snapshot keyed by version).
export type NamedSnapshot = {
  id: number;
  label: string;
  timestamp: string;
  size: number;
};
export type ProjectSnapshot = {
  timestamp: string;
  version: number;
  updates: number[];
};

export type ProjectDocument = {
  id: string;
  timestamp: string;
  version: number;
  updates: number[];
};

export type PreviewSnapshot = {
  id: string;
  timestamp: string;
  version: number;
  updates: number[];
};

export type RollbackProject = {
  projectDocument?: ProjectDocument;
} & ApiResponse;

export type SaveSnapshot = {
  saveSnapshot: boolean;
} & ApiResponse;
