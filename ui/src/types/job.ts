import type { ApiResponse } from "./api";

export type NodeStatus =
  | "pending"
  | "starting"
  | "processing"
  | "completed"
  | "failed";

export type NodeExecution = {
  id: string;
  nodeId: string;
  jobId: string;
  status?: NodeStatus;
  startedAt?: string;
  completedAt?: string;
};

export type JobStatus =
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export type Job = {
  id: string;
  deploymentId?: string;
  deploymentDescription?: string;
  workspaceId: string;
  status: JobStatus;
  startedAt: string;
  completedAt: string;
  outputURLs?: string[];
  logsURL?: string;
};

export type CancelJob = {
  job?: Job;
} & ApiResponse;
