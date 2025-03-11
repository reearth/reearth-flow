import { ApiResponse } from "./api";

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

export type RollbackProject = {
  projectDocument?: ProjectDocument;
} & ApiResponse;
