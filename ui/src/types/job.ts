import type { ApiResponse } from "./api";
import type { Variable } from "./trigger";

export enum JobOrderBy {
  CompletedAt = "completedAt",
  StartedAt = "startedAt",
  Status = "status",
}

export type JobStatus =
  "queued" | "running" | "completed" | "failed" | "cancelled";

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
  variables?: Variable[];
  // The user who started the job. Unset for runs no user started, such as
  // scheduled and API trigger runs, and for jobs from before it was recorded.
  runById?: string;
  // The trigger that started the job. The description is unset when the
  // trigger has since been deleted.
  triggerId?: string;
  triggerDescription?: string;
  origin?: JobOrigin;
};

export type CmsItemJobOrigin = {
  projectId: string;
  modelId: string;
  itemId: string;
  connectionId: string;
  projectName: string;
  modelName: string;
  itemTitle: string;
};

export type JobOrigin = {
  cmsItem?: CmsItemJobOrigin;
};

export type CancelJob = {
  job?: Job;
} & ApiResponse;
