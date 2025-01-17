import type {
  DeploymentFragment,
  ProjectFragment,
  JobFragment,
  JobStatus as GraphqlJobStatus,
  TriggerFragment,
} from "@flow/lib/gql/__gen__/plugins/graphql-request";
import {
  type Deployment,
  type Job,
  type JobStatus,
  type Project,
  type Trigger,
} from "@flow/types";
import { formatDate } from "@flow/utils";

export const toProject = (project: ProjectFragment): Project => ({
  id: project.id,
  name: project.name,
  createdAt: formatDate(project.createdAt),
  updatedAt: formatDate(project.updatedAt),
  description: project.description,
  workspaceId: project.workspaceId,
  deployment: project.deployment ? toDeployment(project.deployment) : undefined,
});

export const toDeployment = (deployment: DeploymentFragment): Deployment => ({
  id: deployment.id,
  workspaceId: deployment.workspaceId,
  projectId: deployment.projectId,
  projectName: deployment.project?.name,
  workflowUrl: deployment.workflowUrl,
  description: deployment.description ?? undefined,
  version: deployment.version,
  createdAt: formatDate(deployment.createdAt),
  updatedAt: formatDate(deployment.updatedAt),
});

export const toTrigger = (trigger: TriggerFragment): Trigger => ({
  id: trigger.id,
  deploymentId: trigger.deploymentId,
  deployment: toDeployment(trigger.deployment),
  workspaceId: trigger.workspaceId,
  createdAt: trigger.createdAt,
  updatedAt: trigger.updatedAt,
  eventSource: trigger.eventSource,
  authToken: trigger.authToken ?? undefined,
  timeInterval: trigger.timeInterval ?? undefined,
});

export const toJob = (job: JobFragment): Job => ({
  id: job.id,
  deploymentId: job.deploymentId,
  workspaceId: job.workspaceId,
  status: toJobStatus(job.status),
  startedAt: job.startedAt,
  completedAt: job.completedAt,
});

export const toJobStatus = (status: GraphqlJobStatus): JobStatus => {
  switch (status) {
    case "RUNNING":
      return "running";
    case "COMPLETED":
      return "completed";
    case "FAILED":
      return "failed";
    case "PENDING":
    default:
      return "queued";
  }
};
