import type {
  DeploymentFragment,
  ProjectFragment,
  JobFragment,
  JobStatus as GraphqlJobStatus,
  TriggerFragment,
  LogFragment,
  ProjectSnapshotFragment,
  ProjectDocumentFragment,
} from "@flow/lib/gql/__gen__/plugins/graphql-request";
import {
  Log,
  type Deployment,
  type Job,
  type JobStatus,
  type Project,
  type Trigger,
  type ProjectSnapshot,
  ProjectDocument,
} from "@flow/types";
import { formatDate } from "@flow/utils";

export const toProject = (project: ProjectFragment): Project => ({
  id: project.id,
  name: project.name,
  version: project.version,
  createdAt: formatDate(project.createdAt),
  updatedAt: formatDate(project.updatedAt),
  description: project.description,
  workspaceId: project.workspaceId,
  sharedToken: project.sharedToken ?? undefined,
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
  description: trigger.description ?? undefined,
});

export const toJob = (job: JobFragment): Job => ({
  id: job.id,
  deploymentId: job.deployment?.id,
  deploymentDescription: job.deployment?.description,
  workspaceId: job.workspaceId,
  status: toJobStatus(job.status),
  startedAt: job.startedAt,
  completedAt: job.completedAt,
  logsURL: job.logsURL ?? undefined,
  outputURLs: job.outputURLs ?? undefined,
});

export const toLog = (log: LogFragment): Log => ({
  nodeId: log.nodeId,
  jobId: log.jobId,
  timestamp: log.timestamp,
  status: log.logLevel,
  message: log.message,
});

export const toProjectSnapShot = (
  projectSnapshot: ProjectSnapshotFragment,
): ProjectSnapshot => ({
  timestamp: projectSnapshot.timestamp,
  version: projectSnapshot.version,
  updates: projectSnapshot.updates,
});

export const toProjectDocument = (
  projectDocument: ProjectDocumentFragment,
): ProjectDocument => ({
  id: projectDocument.id,
  timestamp: projectDocument.timestamp,
  version: projectDocument.version,
  updates: projectDocument.updates,
});

export const toJobStatus = (status: GraphqlJobStatus): JobStatus => {
  switch (status) {
    case "RUNNING":
      return "running";
    case "COMPLETED":
      return "completed";
    case "FAILED":
      return "failed";
    case "CANCELLED":
      return "cancelled";
    case "PENDING":
    default:
      return "queued";
  }
};
