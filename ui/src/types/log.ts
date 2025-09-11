import { ApiResponse } from "./api";

export enum LogLevel {
  Debug = "DEBUG",
  Error = "ERROR",
  Info = "INFO",
  Trace = "TRACE",
  Warn = "WARN",
}

export enum FacingLogLevel {
  Error = "ERROR",
  Info = "INFO",
  Success = "SUCCESS",
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
  nodeId?: string;
  nodeName?: string;
  level: FacingLogLevel;
  message: string;
  metadata?: Record<string, any> | null | undefined;
};

export type GetLogs = {
  logs?: Log[];
} & ApiResponse;
