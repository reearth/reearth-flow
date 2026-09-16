import type { ApiResponse } from "./api";
import type { Diagnostic } from "./diagnostic";

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
  /**
   * Terminal per-node failures. Persisted at job completion, so this is never
   * populated while the job is still running.
   */
  failedNodes?: Diagnostic[];
  /** Diagnostics the engine had to drop rather than emit, if any. */
  droppedEventCount?: number;
};

export type CancelJob = {
  job?: Job;
} & ApiResponse;
