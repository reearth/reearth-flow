import { ApiResponse } from "./api";

export type LogLevel = "info" | "debug" | "warn" | "trace" | "error";

export type Log = {
  // id: string;
  nodeId?: string | null | undefined;
  // workflowId: string;
  jobId: string;
  timestamp: string;
  status: LogLevel;
  message: string;
};

export type GetLogs = {
  logs?: Log[];
} & ApiResponse;
// 2025-03-06T05:40:43.143502792Z
