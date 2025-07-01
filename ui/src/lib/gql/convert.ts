import type {
  AssetFragment,
  DeploymentFragment,
  ProjectFragment,
  WorkspaceFragment,
  ProjectSnapshotFragment,
  JobFragment,
  JobStatus as GraphqlJobStatus,
  NodeStatus as GraphqlNodeStatus,
  TriggerFragment,
  LogFragment,
  ProjectDocumentFragment,
  NodeExecutionFragment,
  ProjectSnapshotMetadataFragment,
} from "@flow/lib/gql/__gen__/plugins/graphql-request";
import type {
  Log,
  Deployment,
  Job,
  JobStatus,
  Project,
  Trigger,
  ProjectDocument,
  NodeExecution,
  NodeStatus,
  ProjectSnapshotMeta,
  ProjectSnapshot,
  Workspace,
  Member,
  Asset,
} from "@flow/types";
import { formatDate, formatFileSize } from "@flow/utils";

export const toProject = (project: ProjectFragment): Project => ({
  id: project.id,
  name: project.name,
  createdAt: formatDate(project.createdAt),
  updatedAt: formatDate(project.updatedAt),
  description: project.description,
  workspaceId: project.workspaceId,
  sharedToken: project.sharedToken ?? undefined,
  deployment: project.deployment ? toDeployment(project.deployment) : undefined,
});

export const toWorkspace = (workspace: WorkspaceFragment): Workspace => ({
  id: workspace.id,
  name: workspace.name,
  personal: workspace.personal,
  members: workspace.members.map(
    (m): Member => ({
      userId: m.userId,
      role: m.role,
      user: m.user
        ? {
            id: m.user?.id,
            name: m.user?.name,
            email: m.user?.email,
          }
        : undefined,
    }),
  ),
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

export const toNodeExecution = (
  node: NodeExecutionFragment,
): NodeExecution => ({
  id: node.id,
  jobId: node.jobId,
  nodeId: node.nodeId,
  status: toNodeStatus(node.status),
  startedAt: node.startedAt,
  completedAt: node.completedAt,
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

export const toProjectSnapShotMeta = (
  projectSnapshot: ProjectSnapshotMetadataFragment,
): ProjectSnapshotMeta => ({
  timestamp: projectSnapshot.timestamp,
  version: projectSnapshot.version,
});

export const toProjectDocument = (
  projectDocument: ProjectDocumentFragment,
): ProjectDocument => ({
  id: projectDocument.id,
  timestamp: projectDocument.timestamp,
  version: projectDocument.version,
  updates: projectDocument.updates,
});

export const toProjectSnapShot = (
  projectSnapshot: ProjectSnapshotFragment,
): ProjectSnapshot => ({
  timestamp: projectSnapshot.timestamp,
  version: projectSnapshot.version,
  updates: projectSnapshot.updates,
});

export const toAsset = (asset: AssetFragment): Asset => ({
  id: asset.id,
  name: asset.name,
  workspaceId: asset.workspaceId,
  createdAt: formatDate(asset.createdAt),
  contentType: asset.contentType,
  size: formatFileSize(asset.size),
  url: asset.url,
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

export const toNodeStatus = (
  status: GraphqlNodeStatus,
): NodeStatus | undefined => {
  switch (status) {
    case "STARTING":
      return "starting";
    case "PENDING":
      return "pending";
    case "PROCESSING":
      return "processing";
    case "COMPLETED":
      return "completed";
    case "FAILED":
      return "failed";
    default:
      return undefined;
  }
};
