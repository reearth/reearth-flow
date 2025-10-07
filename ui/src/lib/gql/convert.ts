import {
  type DeploymentFragment,
  type ProjectFragment,
  type JobFragment,
  type JobStatus as GraphqlJobStatus,
  type NodeStatus as GraphqlNodeStatus,
  type ArchiveExtractionStatus as GraphqlArchiveExtractionStatus,
  type TriggerFragment,
  type ProjectDocumentFragment,
  type NodeExecutionFragment,
  type ProjectSnapshotMetadataFragment,
  type ParameterFragment,
  type AssetFragment,
  type WorkspaceFragment,
  type CmsProjectFragment,
  type CmsModelFragment,
  type CmsItemFragment,
  type CmsAssetFragment,
  type CmsVisibility as GraphqlCmsVisibility,
  type CmsSchemaFieldType as GraphQlCmsSchemaFieldType,
  type UserFacingLogLevel as GraphqlUserFacingLogLevel,
  ParameterType,
  ProjectSnapshotFragment,
} from "@flow/lib/gql/__gen__/plugins/graphql-request";
import type {
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
  AnyProjectVariable,
  Workspace,
  Member,
  Asset,
  ProjectSnapshot,
  ArchiveExtractionStatus,
  CmsProject,
  CmsVisibility,
  CmsModel,
  CmsItem,
  CmsSchemaField,
  CmsSchemaFieldType,
  CmsAsset,
  UserFacingLog,
} from "@flow/types";
import { UserFacingLogLevel } from "@flow/types";
import { formatDate, formatFileSize } from "@flow/utils";

import { UserFacingLogFragment } from "./__gen__/graphql";

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
  userFacingLogsURL: job.userFacingLogsURL ?? undefined,
});

export const toUserFacingLog = (log: UserFacingLogFragment): UserFacingLog => ({
  jobId: log.jobId,
  timestamp: log.timestamp,
  message: log.message,
  nodeId: log.nodeId ?? undefined,
  nodeName: log.nodeName ?? undefined,
  level: toUserFacingLogLevel(log.level),
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
  workspaceId: asset.workspaceId,
  createdAt: formatDate(asset.createdAt),
  fileName: asset.fileName,
  size: formatFileSize(asset.size),
  contentType: asset.contentType,
  name: asset.name,
  url: asset.url,
  uuid: asset.uuid,
  flatFiles: asset.flatFiles,
  public: asset.public,
  archiveExtractionStatus: asset.archiveExtractionStatus
    ? toArchiveExtractionStatus(asset.archiveExtractionStatus)
    : "pending",
});

export const toCmsProject = (cmsProject: CmsProjectFragment): CmsProject => ({
  id: cmsProject.id,
  workspaceId: cmsProject.workspaceId,
  name: cmsProject.name,
  alias: cmsProject.alias,
  description: cmsProject.description ?? undefined,
  license: cmsProject.license ?? undefined,
  readme: cmsProject.readme ?? undefined,
  visibility: cmsProject.visibility
    ? toCmsVisibility(cmsProject.visibility)
    : "public",
  createdAt: formatDate(cmsProject.createdAt),
  updatedAt: formatDate(cmsProject.updatedAt),
});

export const toCmsModel = (cmsModel: CmsModelFragment): CmsModel => ({
  id: cmsModel.id,
  projectId: cmsModel.projectId,
  name: cmsModel.name,
  description: cmsModel.description,
  key: cmsModel.key,
  editorUrl: cmsModel.editorUrl,
  publicApiEp: cmsModel.publicApiEp,
  schema: {
    schemaId: cmsModel.schema.schemaId,
    fields: cmsModel.schema.fields.map(
      (field): CmsSchemaField => ({
        fieldId: field.fieldId,
        name: field.name,
        type: field.type ? toCmsSchemaFieldType(field.type) : "text",
        key: field.key,
        description: field.description ?? undefined,
      }),
    ),
  },
  createdAt: formatDate(cmsModel.createdAt),
  updatedAt: formatDate(cmsModel.updatedAt),
});

export const toCmsItem = (cmsItem: CmsItemFragment): CmsItem => ({
  id: cmsItem.id,
  fields: cmsItem.fields,
  createdAt: formatDate(cmsItem.createdAt),
  updatedAt: formatDate(cmsItem.updatedAt),
});

export const toCmsAsset = (cmsAsset: CmsAssetFragment): CmsAsset => ({
  id: cmsAsset.id,
  uuid: cmsAsset.uuid,
  projectId: cmsAsset.projectId,
  filename: cmsAsset.filename,
  size: cmsAsset.size,
  previewType: cmsAsset.previewType ?? undefined,
  url: cmsAsset.url,
  archiveExtractionStatus: cmsAsset.archiveExtractionStatus ?? undefined,
  public: cmsAsset.public,
  createdAt: formatDate(cmsAsset.createdAt),
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

export const toUserFacingLogLevel = (
  level: GraphqlUserFacingLogLevel,
): UserFacingLogLevel => {
  switch (level) {
    case "ERROR":
      return UserFacingLogLevel.Error;
    case "INFO":
      return UserFacingLogLevel.Info;
    case "SUCCESS":
      return UserFacingLogLevel.Success;
    default:
      return UserFacingLogLevel.Info;
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
): AnyProjectVariable => ({
  id: parameter.id,
  name: parameter.name,
  type: toUserParamVarType(parameter.type),
  defaultValue: parameter.defaultValue,
  required: parameter.required,
  public: parameter.public,
  config: parameter.config,
  createdAt: parameter.createdAt,
  updatedAt: parameter.updatedAt,
  projectId: parameter.projectId,
  // description: parameter.description,
});

export const toUserParamVarType = (type: ParameterType): VarType => {
  switch (type) {
    case "ARRAY":
      return "array";
    case "CHOICE":
      return "choice";
    case "COLOR":
      return "color";
    case "NUMBER":
      return "number";
    case "TEXT":
      return "text";
    case "YES_NO":
      return "yes_no";
    case "DATETIME":
      return "datetime";
    // case "GEOMETRY":
    //   return "geometry";
    // case "MESSAGE":
    //   return "message";
    // case "ATTRIBUTE_NAME":
    //   return "attribute_name";
    // case "COORDINATE_SYSTEM":
    //   return "coordinate_system";
    // case "DATABASE_CONNECTION":
    //   return "database_connection";
    // case "PASSWORD":
    //   return "password";
    // case "REPROJECTION_FILE":
    //   return "reprojection_file";
    // case "WEB_CONNECTION":
    //   return "web_connection";
    default:
      return "unsupported";
  }
};

export const toGqlParameterType = (
  type: VarType,
): ParameterType | undefined => {
  switch (type) {
    case "array":
      return ParameterType.Array;
    case "choice":
      return ParameterType.Choice;
    case "color":
      return ParameterType.Color;
    case "datetime":
      return ParameterType.Datetime;
    case "number":
      return ParameterType.Number;
    case "text":
      return ParameterType.Text;
    case "yes_no":
      return ParameterType.YesNo;
    // case "coordinate_system":
    //   return ParameterType.CoordinateSystem;
    // case "attribute_name":
    //   return ParameterType.AttributeName;
    // case "database_connection":
    //   return ParameterType.DatabaseConnection;
    // case "geometry":
    //   return ParameterType.Geometry;
    // case "message":
    //   return ParameterType.Message;
    // case "password":
    //   return ParameterType.Password;
    // case "reprojection_file":
    //   return ParameterType.ReprojectionFile;
    // case "web_connection":
    //   return ParameterType.WebConnection;
    default:
      return undefined;
  }
};

export const toArchiveExtractionStatus = (
  status: GraphqlArchiveExtractionStatus,
): ArchiveExtractionStatus => {
  switch (status) {
    case "DONE":
      return "done";
    case "FAILED":
      return "failed";
    case "IN_PROGRESS":
      return "in_progress";
    case "SKIPPED":
      return "skipped";
    case "PENDING":
    default:
      return "pending";
  }
};

export const toCmsVisibility = (
  visibility: GraphqlCmsVisibility,
): CmsVisibility => {
  switch (visibility) {
    case "PRIVATE":
      return "private";
    default:
      return "public";
  }
};

export const toCmsSchemaFieldType = (
  type: GraphQlCmsSchemaFieldType,
): CmsSchemaFieldType => {
  switch (type) {
    case "TEXTAREA":
      return "text_area";
    case "RICHTEXT":
      return "rich_text";
    case "MARKDOWNTEXT":
      return "mark_down_text";
    case "ASSET":
      return "asset";
    case "DATE":
      return "date";
    case "BOOL":
      return "bool";
    case "SELECT":
      return "select";
    case "TAG":
      return "tag";
    case "INTEGER":
      return "integer";
    case "NUMBER":
      return "number";
    case "REFERENCE":
      return "reference";
    case "CHECKBOX":
      return "checkbox";
    case "URL":
      return "url";
    case "GROUP":
      return "group";
    case "GEOMETRYOBJECT":
      return "geometry_object";
    case "GEOMETRYEDITOR":
      return "geometry_editor";
    default:
      return "text";
  }
};
