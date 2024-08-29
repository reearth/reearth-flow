import { Project } from "./project";

export type Run = {
  id: string;
  project: Pick<Project, "id" | "name" | "workflows" | "createdAt">;
  // projectId: string;
  // projectRevisionId: string; OR projectVersionId: string; OR workflowId: string;
  status: "running" | "queued" | "completed" | "failed";
  startedAt: string;
  completedAt?: string;
  logs?: unknown; // or boolean? or logId?
  ranBy?: string;
  trigger?: "api" | "cms" | "manual"; // do we need CMS? Or maybe "api" and "native" is enough? or..?
};

// Do we need a RunResult type?
// How do we append NodeResults? Does order matter?
