export type JobStatus = "pending" | "running" | "completed" | "failed";

export type Job = {
  id: string;
  deploymentId: string;
  workspaceId: string;
  status: JobStatus;
  startedAt: string;
  completedAt: string;
  // deployment: Deployment;
  // workspace: Workspace;
};
