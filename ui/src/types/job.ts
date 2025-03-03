import type { ApiResponse } from "./api";
import type { NodeExecution } from "./nodeExecutions";

export type JobStatus =
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export type Job = {
  id: string;
  deploymentId: string;
  workspaceId: string;
  status: JobStatus;
  startedAt: string;
  completedAt: string;
  outputURLs?: string[];
  logsURL?: string;
  nodeExecutions?: NodeExecution[];
};

export type CancelJob = {
  job?: Job;
} & ApiResponse;
