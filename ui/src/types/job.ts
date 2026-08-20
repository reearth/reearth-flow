import type { ApiResponse } from "./api";

export enum JobOrderBy {
  CompletedAt = "completedAt",
  StartedAt = "startedAt",
  Status = "status",
}

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
  userFacingLogsURL?: string;
};

export type CancelJob = {
  job?: Job;
} & ApiResponse;
