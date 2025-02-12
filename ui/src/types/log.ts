import { ApiResponse } from "./api";

export enum LogLevel {
  ERROR = "ERROR",
  WARN = "WARN",
  INFO = "INFO",
  DEBUG = "DEBUG",
  TRACE = "TRACE",
}

export type Log = {
  // id: string;
  workflowId: string;
  jobId: string;
  nodeId?: string;
  ts: string;
  level: LogLevel;
  msg: string;
};

export type GetLogs = {
  Logs?: Log[];
} & ApiResponse;
