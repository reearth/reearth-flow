import { ApiResponse } from "./api";

export type ProjectSnapshotMeta = {
  timestamp: string;
  version: number;
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
