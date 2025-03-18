import type { ApiResponse } from "./api";

export type NodeStatus = "pending" | "running" | "succeeded" | "failed";

export type EdgeStatus = "inProgress" | "completed" | "failed";

export type EdgeExecution = {
  id: string;
  edgeId: string;
  jobId: string;
  status?: EdgeStatus;
  createdAt?: string;
  startedAt?: string;
  completedAt?: string;
  featureId?: string;
  intermediateDataUrl?: string;
};

export type NodeExecution = {
  nodeId: string;
  status: NodeStatus;
  startedAt?: string;
  completedAt?: string;
  intermediateDataUrl?: string;
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
