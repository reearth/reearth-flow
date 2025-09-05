import { ApiResponse } from "./api";

export enum LogLevel {
  Debug = "DEBUG",
  Error = "ERROR",
  Info = "INFO",
  Trace = "TRACE",
  Warn = "WARN",
}

export type Log = {
  nodeId?: string | null | undefined;
  jobId: string;
  timestamp: string;
  status: LogLevel;
  message: string;
};

export type FacingLog = {
  jobId: string;
  timestamp: string;
  message: string;
  metadata?: Record<string, any> | null | undefined;
};

export type GetLogs = {
  logs?: Log[];
} & ApiResponse;
