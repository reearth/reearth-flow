import {
  type DeploymentFragment,
  type ProjectFragment,
  type JobFragment,
  type JobStatus as GraphqlJobStatus,
  type NodeStatus as GraphqlNodeStatus,
  type TriggerFragment,
  type LogFragment,
  type ProjectDocumentFragment,
  type NodeExecutionFragment,
  type ProjectSnapshotMetadataFragment,
  type ParameterFragment,
  ParameterType,
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
  VarType,
  ProjectVariable,
  Workspace,
  Member,
} from "@flow/types";
import { formatDate } from "@flow/utils";

import { WorkspaceFragment } from "./__gen__/graphql";

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

export const toProjectVariable = (
  parameter: ParameterFragment,
): ProjectVariable => ({
  id: parameter.id,
  name: parameter.name,
  type: toUserParamVarType(parameter.type),
  defaultValue: parameter.defaultValue,
  required: parameter.required,
  public: parameter.public,
  createdAt: parameter.createdAt,
  updatedAt: parameter.updatedAt,
  projectId: parameter.projectId,
  // description: parameter.description,
});

export const toUserParamVarType = (type: ParameterType): VarType => {
  switch (type) {
    case "ATTRIBUTE_NAME":
      return "attribute_name";
    case "CHOICE":
      return "choice";
    case "COLOR":
      return "color";
    case "COORDINATE_SYSTEM":
      return "coordinate_system";
    case "DATABASE_CONNECTION":
      return "database_connection";
    case "DATETIME":
      return "datetime";
    case "FILE_FOLDER":
      return "file_folder";
    case "GEOMETRY":
      return "geometry";
    case "MESSAGE":
      return "message";
    case "NUMBER":
      return "number";
    case "PASSWORD":
      return "password";
    case "REPROJECTION_FILE":
      return "reprojection_file";
    case "TEXT":
      return "text";
    case "WEB_CONNECTION":
      return "web_connection";
    case "YES_NO":
      return "yes_no";
    default:
      return "unsupported";
  }
};

export const toGqlParameterType = (
  type: VarType,
): ParameterType | undefined => {
  switch (type) {
    case "attribute_name":
      return ParameterType.AttributeName;
    case "choice":
      return ParameterType.Choice;
    case "color":
      return ParameterType.Color;
    case "coordinate_system":
      return ParameterType.CoordinateSystem;
    case "database_connection":
      return ParameterType.DatabaseConnection;
    case "datetime":
      return ParameterType.Datetime;
    case "file_folder":
      return ParameterType.FileFolder;
    case "geometry":
      return ParameterType.Geometry;
    case "message":
      return ParameterType.Message;
    case "number":
      return ParameterType.Number;
    case "password":
      return ParameterType.Password;
    case "reprojection_file":
      return ParameterType.ReprojectionFile;
    case "text":
      return ParameterType.Text;
    case "web_connection":
      return ParameterType.WebConnection;
    case "yes_no":
      return ParameterType.YesNo;
    default:
      return undefined;
  }
};
