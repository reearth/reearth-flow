import { Deployment } from "./deployment";
// import { Project } from "./project";

export type JobStatus = "queued" | "running" | "completed" | "failed";

// export type Trigger = "api" | "cms" | "manual"; // do we need CMS? Or maybe "api" and "native" is enough? or..?

export type Job = {
  id: string;
  deploymentId: string;
  workspaceId: string;
  status: JobStatus;
  startedAt: string;
  completedAt: string;
  logs?: unknown; // or boolean? or logId?
  // trigger?: Trigger;
  deployment?: Deployment;
  // workspace: Workspace;
};

// type Run = {
//   id: string;
//   project: Pick<Project, "id" | "name" | "workflows" | "createdAt">;
//   // projectId: string;
//   // projectRevisionId: string; OR projectVersionId: string; OR workflowId: string;
//   status: "running" | "queued" | "completed" | "failed";
//   startedAt: string;
//   completedAt?: string;
//   logs?: unknown; // or boolean? or logId?
//   ranBy?: string;
//   trigger?: Trigger;
// };

// Do we need a RunResult type?
// How do we append NodeResults? Does order matter?
