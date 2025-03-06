import { ApiResponse } from "./api";

export enum LogLevel {
  Debug = "DEBUG",
  Error = "ERROR",
  Info = "INFO",
  Trace = "TRACE",
  Warn = "WARN",
}

export type Log = {
  // id: string;
  nodeId?: string;
  // workflowId: string;
  jobId: string;
  timeStamp: string;
  status: LogLevel;
  message: string;
};

// export type LiveLog =
//   | {
//       __typename?: "Log";
//       jobId: string;
//       nodeId?: string | null;
//       timestamp: string;
//       logLevel: LogLevel;
//       message: string;
//     }
//   | null
//   | undefined;

export type GetLogs = {
  Logs?: Log[];
} & ApiResponse;
