/** Internal type. DO NOT USE DIRECTLY. */
type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
/** Internal type. DO NOT USE DIRECTLY. */
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
import { GraphQLClient, RequestOptions } from 'graphql-request';
import gql from 'graphql-tag';
type GraphQLClientRequestHeaders = RequestOptions['requestHeaders'];
export type ApiDriverInput = {
  token: string;
};

export type AddMemberToWorkspaceInput = {
  role: Role;
  userId: string | number;
  workspaceId: string | number;
};

export type ArchiveExtractionStatus =
  | 'DONE'
  | 'FAILED'
  | 'IN_PROGRESS'
  | 'PENDING'
  | 'SKIPPED';

export type CmsExportType =
  | 'GEOJSON'
  | 'JSON';

export type CmsSchemaFieldType =
  | 'ASSET'
  | 'BOOL'
  | 'CHECKBOX'
  | 'DATE'
  | 'GEOMETRYEDITOR'
  | 'GEOMETRYOBJECT'
  | 'GROUP'
  | 'INTEGER'
  | 'MARKDOWNTEXT'
  | 'NUMBER'
  | 'REFERENCE'
  | 'RICHTEXT'
  | 'SELECT'
  | 'TAG'
  | 'TEXT'
  | 'TEXTAREA'
  | 'URL';

export type CmsVisibility =
  | 'PRIVATE'
  | 'PUBLIC';

export type CancelJobInput = {
  jobId: string | number;
};

export type CreateAssetInput = {
  file?: any;
  name?: string | null | undefined;
  token?: string | null | undefined;
  workspaceId: string | number;
};

export type CreateAssetUploadInput = {
  contentEncoding?: string | null | undefined;
  contentLength?: number | null | undefined;
  cursor?: string | null | undefined;
  filename?: string | null | undefined;
  workspaceId: string | number;
};

export type CreateDeploymentInput = {
  description: string;
  file: any;
  projectId?: string | number | null | undefined;
  workspaceId: string | number;
};

export type CreateProjectInput = {
  archived?: boolean | null | undefined;
  description?: string | null | undefined;
  name?: string | null | undefined;
  workspaceId: string | number;
};

export type CreateTriggerInput = {
  apiDriverInput?: ApiDriverInput | null | undefined;
  deploymentId: string | number;
  description: string;
  enabled: boolean;
  timeDriverInput?: TimeDriverInput | null | undefined;
  variables?: Array<VariableInput> | null | undefined;
  workspaceId: string | number;
};

export type CreateWorkspaceInput = {
  name: string;
};

export type DeclareParameterInput = {
  config?: any;
  defaultValue?: any;
  index?: number | null | undefined;
  name: string;
  public: boolean;
  required: boolean;
  type: ParameterType;
};

export type DeleteAssetInput = {
  assetId: string | number;
};

export type DeleteDeploymentInput = {
  deploymentId: string | number;
};

export type DeleteProjectInput = {
  projectId: string | number;
};

export type DeleteWorkspaceInput = {
  workspaceId: string | number;
};

export type EventSourceType =
  | 'API_DRIVEN'
  | 'TIME_DRIVEN';

export type ExecuteDeploymentInput = {
  deploymentId: string | number;
};

export type GetHeadInput = {
  projectId?: string | number | null | undefined;
  workspaceId: string | number;
};

export type JobStatus =
  | 'CANCELLED'
  | 'COMPLETED'
  | 'FAILED'
  | 'PENDING'
  | 'RUNNING';

export type NodeStatus =
  | 'COMPLETED'
  | 'FAILED'
  | 'PENDING'
  | 'PROCESSING'
  | 'STARTING';

export type OrderDirection =
  | 'ASC'
  | 'DESC';

export type PageBasedPagination = {
  orderBy?: string | null | undefined;
  orderDir?: OrderDirection | null | undefined;
  page: number;
  pageSize: number;
};

export type ParameterBatchInput = {
  creates?: Array<DeclareParameterInput> | null | undefined;
  deletes?: Array<string | number> | null | undefined;
  projectId: string | number;
  reorders?: Array<UpdateParameterOrderInput> | null | undefined;
  updates?: Array<ParameterUpdateItem> | null | undefined;
};

export type ParameterType =
  | 'ARRAY'
  | 'CHOICE'
  | 'COLOR'
  | 'DATETIME'
  | 'FILE_FOLDER'
  | 'NUMBER'
  | 'TEXT'
  | 'YES_NO';

export type ParameterUpdateItem = {
  config?: any;
  defaultValue?: any;
  name: string;
  paramId: string | number;
  public: boolean;
  required: boolean;
  type: ParameterType;
};

export type RemoveMemberFromWorkspaceInput = {
  userId: string | number;
  workspaceId: string | number;
};

export type RemoveParameterInput = {
  paramId: string | number;
};

export type RemoveParametersInput = {
  paramIds: Array<string | number>;
};

export type Role =
  | 'maintainer'
  | 'owner'
  | 'reader'
  | 'writer';

export type RunParameterInput = {
  config?: any;
  id: string | number;
  index: number;
  name: string;
  public: boolean;
  required: boolean;
  type: ParameterType;
  value: any;
};

export type RunProjectInput = {
  file: any;
  parameters?: Array<RunParameterInput> | null | undefined;
  previousJobId?: string | number | null | undefined;
  projectId: string | number;
  startNodeId?: string | number | null | undefined;
  workspaceId: string | number;
};

export type ShareProjectInput = {
  projectId: string | number;
};

export type TimeDriverInput = {
  interval: TimeInterval;
};

export type TimeInterval =
  | 'EVERY_DAY'
  | 'EVERY_HOUR'
  | 'EVERY_MONTH'
  | 'EVERY_WEEK';

export type UnshareProjectInput = {
  projectId: string | number;
};

export type UpdateAssetInput = {
  assetId: string | number;
  name?: string | null | undefined;
};

export type UpdateDeploymentInput = {
  deploymentId: string | number;
  description?: string | null | undefined;
  file?: any;
};

export type UpdateMeInput = {
  email?: string | null | undefined;
  lang?: any;
  name?: string | null | undefined;
  password?: string | null | undefined;
  passwordConfirmation?: string | null | undefined;
};

export type UpdateMemberOfWorkspaceInput = {
  role: Role;
  userId: string | number;
  workspaceId: string | number;
};

export type UpdateParameterInput = {
  config?: any;
  defaultValue?: any;
  name: string;
  public: boolean;
  required: boolean;
  type: ParameterType;
};

export type UpdateParameterOrderInput = {
  newIndex: number;
  paramId: string | number;
};

export type UpdateProjectInput = {
  archived?: boolean | null | undefined;
  basicAuthPassword?: string | null | undefined;
  basicAuthUsername?: string | null | undefined;
  description?: string | null | undefined;
  isBasicAuthActive?: boolean | null | undefined;
  isLocked?: boolean | null | undefined;
  name?: string | null | undefined;
  projectId: string | number;
};

export type UpdateTriggerInput = {
  apiDriverInput?: ApiDriverInput | null | undefined;
  deploymentId?: string | number | null | undefined;
  description?: string | null | undefined;
  enabled?: boolean | null | undefined;
  timeDriverInput?: TimeDriverInput | null | undefined;
  triggerId: string | number;
  variables?: Array<VariableInput> | null | undefined;
};

export type UpdateWorkerConfigInput = {
  bootDiskSizeGB?: number | null | undefined;
  channelBufferSize?: number | null | undefined;
  computeCpuMilli?: number | null | undefined;
  computeMemoryMib?: number | null | undefined;
  featureFlushThreshold?: number | null | undefined;
  machineType?: string | null | undefined;
  maxConcurrency?: number | null | undefined;
  nodeStatusPropagationDelayMilli?: number | null | undefined;
  taskCount?: number | null | undefined;
  threadPoolSize?: number | null | undefined;
};

export type UpdateWorkspaceInput = {
  name: string;
  workspaceId: string | number;
};

export type UserFacingLogLevel =
  | 'ERROR'
  | 'INFO'
  | 'SUCCESS';

export type VariableInput = {
  key: string;
  type: ParameterType;
  value: any;
};

export type GetAssetsQueryVariables = Exact<{
  workspaceId: string;
  keyword?: string | null | undefined;
  pagination: PageBasedPagination;
}>;


export type GetAssetsQuery = { assets: { totalCount: number, nodes: Array<{ id: string, workspaceId: string, createdAt: any, fileName: string, size: any, contentType: string, name: string, url: string, uuid: string, flatFiles: boolean, public: boolean, archiveExtractionStatus: ArchiveExtractionStatus | null } | null>, pageInfo: { totalCount: number, currentPage: number | null, totalPages: number | null } } };

export type CreateAssetMutationVariables = Exact<{
  input: CreateAssetInput;
}>;


export type CreateAssetMutation = { createAsset: { asset: { id: string, workspaceId: string, createdAt: any, fileName: string, size: any, contentType: string, name: string, url: string, uuid: string, flatFiles: boolean, public: boolean, archiveExtractionStatus: ArchiveExtractionStatus | null } } | null };

export type CreateAssetUploadMutationVariables = Exact<{
  input: CreateAssetUploadInput;
}>;


export type CreateAssetUploadMutation = { createAssetUpload: { token: string, url: string, contentType: string | null, contentLength: number, contentEncoding: string | null, next: string | null } | null };

export type UpdateAssetMutationVariables = Exact<{
  input: UpdateAssetInput;
}>;


export type UpdateAssetMutation = { updateAsset: { asset: { id: string, workspaceId: string, createdAt: any, fileName: string, size: any, contentType: string, name: string, url: string, uuid: string, flatFiles: boolean, public: boolean, archiveExtractionStatus: ArchiveExtractionStatus | null } } | null };

export type DeleteAssetMutationVariables = Exact<{
  input: DeleteAssetInput;
}>;


export type DeleteAssetMutation = { deleteAsset: { assetId: string } | null };

export type GetCmsProjectByIdOrAliasQueryVariables = Exact<{
  projectIdOrAlias: string;
}>;


export type GetCmsProjectByIdOrAliasQuery = { cmsProject: { id: string, name: string, alias: string, description: string | null, license: string | null, readme: string | null, workspaceId: string, visibility: CmsVisibility, topics: Array<string>, starCount: number, createdAt: any, updatedAt: any } | null };

export type GetCmsProjectsQueryVariables = Exact<{
  workspaceIds: Array<string> | string;
  keyword?: string | null | undefined;
  publicOnly?: boolean | null | undefined;
  page?: number | null | undefined;
  pageSize?: number | null | undefined;
}>;


export type GetCmsProjectsQuery = { cmsProjects: Array<{ id: string, name: string, alias: string, description: string | null, license: string | null, readme: string | null, workspaceId: string, visibility: CmsVisibility, topics: Array<string>, starCount: number, createdAt: any, updatedAt: any }> };

export type GetCmsModelsQueryVariables = Exact<{
  projectId: string;
  page?: number | null | undefined;
  pageSize?: number | null | undefined;
}>;


export type GetCmsModelsQuery = { cmsModels: { totalCount: number, models: Array<{ id: string, projectId: string, name: string, description: string, editorUrl: string, key: string, publicApiEp: string, createdAt: any, updatedAt: any, schema: { schemaId: string, fields: Array<{ fieldId: string, key: string, type: CmsSchemaFieldType, name: string, description: string | null }> } }> } };

export type GetCmsItemsQueryVariables = Exact<{
  projectId: string;
  modelId: string;
  keyword?: string | null | undefined;
  page?: number | null | undefined;
  pageSize?: number | null | undefined;
}>;


export type GetCmsItemsQuery = { cmsItems: { totalCount: number, items: Array<{ id: string, fields: any, createdAt: any, updatedAt: any }> } };

export type GetCmsAssetQueryVariables = Exact<{
  assetId: string;
}>;


export type GetCmsAssetQuery = { cmsAsset: { id: string, uuid: string, projectId: string, filename: string, size: number, previewType: string | null, url: string, archiveExtractionStatus: string | null, public: boolean, createdAt: any } | null };

export type GetCmsModelExportUrlQueryVariables = Exact<{
  projectId: string;
  modelId: string;
  exportType: CmsExportType;
}>;


export type GetCmsModelExportUrlQuery = { cmsModelExportUrl: string };

export type CreateDeploymentMutationVariables = Exact<{
  input: CreateDeploymentInput;
}>;


export type CreateDeploymentMutation = { createDeployment: { deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } } | null };

export type UpdateDeploymentMutationVariables = Exact<{
  input: UpdateDeploymentInput;
}>;


export type UpdateDeploymentMutation = { updateDeployment: { deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } } | null };

export type DeleteDeploymentMutationVariables = Exact<{
  input: DeleteDeploymentInput;
}>;


export type DeleteDeploymentMutation = { deleteDeployment: { deploymentId: string } | null };

export type ExecuteDeploymentMutationVariables = Exact<{
  input: ExecuteDeploymentInput;
}>;


export type ExecuteDeploymentMutation = { executeDeployment: { job: { id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt: any, outputURLs: Array<string> | null, userFacingLogsURL: string | null, debug: boolean | null, deployment: { id: string, description: string } | null } } | null };

export type GetDeploymentsQueryVariables = Exact<{
  workspaceId: string;
  keyword?: string | null | undefined;
  pagination: PageBasedPagination;
}>;


export type GetDeploymentsQuery = { deployments: { totalCount: number, nodes: Array<{ id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } | null>, pageInfo: { totalCount: number, currentPage: number | null, totalPages: number | null } } };

export type GetDeploymentHeadQueryVariables = Exact<{
  input: GetHeadInput;
}>;


export type GetDeploymentHeadQuery = { deploymentHead: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } | null };

export type GetLatestProjectSnapshotQueryVariables = Exact<{
  projectId: string;
}>;


export type GetLatestProjectSnapshotQuery = { latestProjectSnapshot: { id: string, timestamp: any, updates: Array<number>, version: number } | null };

export type GetProjectSnapshotQueryVariables = Exact<{
  projectId: string;
  version: number;
}>;


export type GetProjectSnapshotQuery = { projectSnapshot: { timestamp: any, updates: Array<number>, version: number } };

export type GetProjectHistoryQueryVariables = Exact<{
  projectId: string;
}>;


export type GetProjectHistoryQuery = { projectHistory: Array<{ timestamp: any, version: number }> };

export type PreviewSnapshotMutationVariables = Exact<{
  projectId: string;
  version: number;
}>;


export type PreviewSnapshotMutation = { previewSnapshot: { id: string, timestamp: any, updates: Array<number>, version: number } | null };

export type RollbackProjectMutationVariables = Exact<{
  projectId: string;
  version: number;
}>;


export type RollbackProjectMutation = { rollbackProject: { id: string, timestamp: any, updates: Array<number>, version: number } | null };

export type SaveSnapshotMutationVariables = Exact<{
  projectId: string;
}>;


export type SaveSnapshotMutation = { saveSnapshot: boolean };

export type ProjectFragment = { id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken: string | null, isLocked: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } | null };

export type WorkspaceFragment = { id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> };

export type ParameterFragment = { id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config: any, createdAt: any, updatedAt: any };

export type DeploymentFragment = { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null };

export type VariableFragment = { key: string, type: ParameterType, value: any };

export type TriggerFragment = { id: string, createdAt: any, updatedAt: any, lastTriggered: any, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken: string | null, timeInterval: TimeInterval | null, description: string, enabled: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null }, variables: Array<{ key: string, type: ParameterType, value: any }> };

export type NodeExecutionFragment = { id: string, nodeId: string, jobId: string, status: NodeStatus, createdAt: any, startedAt: any, completedAt: any };

export type JobFragment = { id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt: any, outputURLs: Array<string> | null, userFacingLogsURL: string | null, debug: boolean | null, deployment: { id: string, description: string } | null };

export type AssetFragment = { id: string, workspaceId: string, createdAt: any, fileName: string, size: any, contentType: string, name: string, url: string, uuid: string, flatFiles: boolean, public: boolean, archiveExtractionStatus: ArchiveExtractionStatus | null };

export type ProjectDocumentFragment = { id: string, timestamp: any, updates: Array<number>, version: number };

export type ProjectSnapshotMetadataFragment = { timestamp: any, version: number };

export type ProjectSnapshotFragment = { timestamp: any, updates: Array<number>, version: number };

export type UserFacingLogFragment = { jobId: string, timestamp: any, nodeId: string | null, nodeName: string | null, level: UserFacingLogLevel, message: string };

export type CmsProjectFragment = { id: string, name: string, alias: string, description: string | null, license: string | null, readme: string | null, workspaceId: string, visibility: CmsVisibility, topics: Array<string>, starCount: number, createdAt: any, updatedAt: any };

export type CmsModelFragment = { id: string, projectId: string, name: string, description: string, editorUrl: string, key: string, publicApiEp: string, createdAt: any, updatedAt: any, schema: { schemaId: string, fields: Array<{ fieldId: string, key: string, type: CmsSchemaFieldType, name: string, description: string | null }> } };

export type CmsItemFragment = { id: string, fields: any, createdAt: any, updatedAt: any };

export type CmsAssetFragment = { id: string, uuid: string, projectId: string, filename: string, size: number, previewType: string | null, url: string, archiveExtractionStatus: string | null, public: boolean, createdAt: any };

export type WorkerConfigFragment = { id: string, machineType: string | null, computeCpuMilli: number | null, computeMemoryMib: number | null, bootDiskSizeGB: number | null, taskCount: number | null, maxConcurrency: number | null, threadPoolSize: number | null, channelBufferSize: number | null, featureFlushThreshold: number | null, nodeStatusPropagationDelayMilli: number | null, createdAt: any, updatedAt: any };

export type GetJobsQueryVariables = Exact<{
  workspaceId: string;
  keyword?: string | null | undefined;
  pagination: PageBasedPagination;
}>;


export type GetJobsQuery = { jobs: { totalCount: number, nodes: Array<{ id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt: any, outputURLs: Array<string> | null, userFacingLogsURL: string | null, debug: boolean | null, deployment: { id: string, description: string } | null } | null>, pageInfo: { totalCount: number, currentPage: number | null, totalPages: number | null } } };

export type GetJobQueryVariables = Exact<{
  id: string;
}>;


export type GetJobQuery = { job: { id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt: any, outputURLs: Array<string> | null, userFacingLogsURL: string | null, debug: boolean | null, deployment: { id: string, description: string } | null } | null };

export type GetNodeExecutionQueryVariables = Exact<{
  jobId: string;
  nodeId: string;
}>;


export type GetNodeExecutionQuery = { nodeExecution: { id: string, nodeId: string, jobId: string, status: NodeStatus, createdAt: any, startedAt: any, completedAt: any } | null };

export type CancelJobMutationVariables = Exact<{
  input: CancelJobInput;
}>;


export type CancelJobMutation = { cancelJob: { job: { id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt: any, outputURLs: Array<string> | null, userFacingLogsURL: string | null, debug: boolean | null, deployment: { id: string, description: string } | null } | null } };

export type CreateProjectMutationVariables = Exact<{
  input: CreateProjectInput;
}>;


export type CreateProjectMutation = { createProject: { project: { id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken: string | null, isLocked: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } | null } } | null };

export type GetProjectsQueryVariables = Exact<{
  workspaceId: string;
  keyword?: string | null | undefined;
  pagination: PageBasedPagination;
}>;


export type GetProjectsQuery = { projects: { totalCount: number, nodes: Array<{ id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken: string | null, isLocked: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } | null } | null>, pageInfo: { totalCount: number, currentPage: number | null, totalPages: number | null } } };

export type GetProjectByIdQueryVariables = Exact<{
  projectId: string;
}>;


export type GetProjectByIdQuery = { node:
    | { __typename: 'Asset' }
    | { __typename: 'Deployment' }
    | { __typename: 'Job' }
    | { __typename: 'NodeExecution' }
    | { __typename: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken: string | null, isLocked: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } | null }
    | { __typename: 'ProjectDocument' }
    | { __typename: 'Trigger' }
    | { __typename: 'User' }
    | { __typename: 'Workspace' }
   | null };

export type UpdateProjectMutationVariables = Exact<{
  input: UpdateProjectInput;
}>;


export type UpdateProjectMutation = { updateProject: { project: { id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken: string | null, isLocked: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } | null } } | null };

export type DeleteProjectMutationVariables = Exact<{
  input: DeleteProjectInput;
}>;


export type DeleteProjectMutation = { deleteProject: { projectId: string } | null };

export type RunProjectMutationVariables = Exact<{
  input: RunProjectInput;
}>;


export type RunProjectMutation = { runProject: { job: { id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt: any, outputURLs: Array<string> | null, userFacingLogsURL: string | null, debug: boolean | null, deployment: { id: string, description: string } | null } } | null };

export type CopyProjectMutationVariables = Exact<{
  projectId: string;
  source: string;
}>;


export type CopyProjectMutation = { copyProject: boolean };

export type ImportProjectMutationVariables = Exact<{
  projectId: string;
  data: any;
}>;


export type ImportProjectMutation = { importProject: boolean };

export type GetSharedProjectQueryVariables = Exact<{
  token: string;
}>;


export type GetSharedProjectQuery = { sharedProject: { project: { id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken: string | null, isLocked: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null } | null } } };

export type GetSharedProjectInfoQueryVariables = Exact<{
  projectId: string;
}>;


export type GetSharedProjectInfoQuery = { projectSharingInfo: { projectId: string, sharingToken: string | null } };

export type ShareProjectMutationVariables = Exact<{
  input: ShareProjectInput;
}>;


export type ShareProjectMutation = { shareProject: { projectId: string, sharingUrl: string } | null };

export type UnshareProjectMutationVariables = Exact<{
  input: UnshareProjectInput;
}>;


export type UnshareProjectMutation = { unshareProject: { projectId: string } | null };

export type OnJobStatusChangeSubscriptionVariables = Exact<{
  jobId: string;
}>;


export type OnJobStatusChangeSubscription = { jobStatus: JobStatus };

export type OnNodeStatusChangeSubscriptionVariables = Exact<{
  jobId: string;
  nodeId: string;
}>;


export type OnNodeStatusChangeSubscription = { nodeStatus: NodeStatus };

export type UserFacingLogsSubscriptionVariables = Exact<{
  jobId: string;
}>;


export type UserFacingLogsSubscription = { userFacingLogs: { jobId: string, timestamp: any, nodeId: string | null, nodeName: string | null, level: UserFacingLogLevel, message: string } | null };

export type CreateTriggerMutationVariables = Exact<{
  input: CreateTriggerInput;
}>;


export type CreateTriggerMutation = { createTrigger: { id: string, createdAt: any, updatedAt: any, lastTriggered: any, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken: string | null, timeInterval: TimeInterval | null, description: string, enabled: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null }, variables: Array<{ key: string, type: ParameterType, value: any }> } };

export type UpdateTriggerMutationVariables = Exact<{
  input: UpdateTriggerInput;
}>;


export type UpdateTriggerMutation = { updateTrigger: { id: string, createdAt: any, updatedAt: any, lastTriggered: any, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken: string | null, timeInterval: TimeInterval | null, description: string, enabled: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null }, variables: Array<{ key: string, type: ParameterType, value: any }> } };

export type DeleteTriggerMutationVariables = Exact<{
  triggerId: string;
}>;


export type DeleteTriggerMutation = { deleteTrigger: boolean };

export type GetTriggersQueryVariables = Exact<{
  workspaceId: string;
  keyword?: string | null | undefined;
  pagination: PageBasedPagination;
}>;


export type GetTriggersQuery = { triggers: { totalCount: number, nodes: Array<{ id: string, createdAt: any, updatedAt: any, lastTriggered: any, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken: string | null, timeInterval: TimeInterval | null, description: string, enabled: boolean, deployment: { id: string, projectId: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project: { name: string } | null }, variables: Array<{ key: string, type: ParameterType, value: any }> } | null>, pageInfo: { totalCount: number, currentPage: number | null, totalPages: number | null } } };

export type GetMeQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMeQuery = { me: { id: string, name: string, email: string, myWorkspaceId: string, lang: any } | null };

export type GetMeAndWorkspacesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMeAndWorkspacesQuery = { me: { id: string, name: string, email: string, myWorkspaceId: string, lang: any, workspaces: Array<{ id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> }> } | null };

export type SearchUserQueryVariables = Exact<{
  email: string;
}>;


export type SearchUserQuery = { searchUser: { id: string, name: string, email: string } | null };

export type UpdateMeMutationVariables = Exact<{
  input: UpdateMeInput;
}>;


export type UpdateMeMutation = { updateMe: { me: { id: string, name: string, email: string, lang: any } } | null };

export type GetWorkerConfigQueryVariables = Exact<{ [key: string]: never; }>;


export type GetWorkerConfigQuery = { workerConfig: { id: string, machineType: string | null, computeCpuMilli: number | null, computeMemoryMib: number | null, bootDiskSizeGB: number | null, taskCount: number | null, maxConcurrency: number | null, threadPoolSize: number | null, channelBufferSize: number | null, featureFlushThreshold: number | null, nodeStatusPropagationDelayMilli: number | null, createdAt: any, updatedAt: any } | null };

export type UpdateWorkerConfigMutationVariables = Exact<{
  input: UpdateWorkerConfigInput;
}>;


export type UpdateWorkerConfigMutation = { updateWorkerConfig: { config: { id: string, machineType: string | null, computeCpuMilli: number | null, computeMemoryMib: number | null, bootDiskSizeGB: number | null, taskCount: number | null, maxConcurrency: number | null, threadPoolSize: number | null, channelBufferSize: number | null, featureFlushThreshold: number | null, nodeStatusPropagationDelayMilli: number | null, createdAt: any, updatedAt: any } } | null };

export type DeleteWorkerConfigMutationVariables = Exact<{ [key: string]: never; }>;


export type DeleteWorkerConfigMutation = { deleteWorkerConfig: { id: string } | null };

export type GetWorkflowParametersQueryVariables = Exact<{
  projectId: string;
}>;


export type GetWorkflowParametersQuery = { parameters: Array<{ id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config: any, createdAt: any, updatedAt: any }> };

export type CreateWorkflowVariableMutationVariables = Exact<{
  projectId: string;
  input: DeclareParameterInput;
}>;


export type CreateWorkflowVariableMutation = { declareParameter: { id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config: any, createdAt: any, updatedAt: any } };

export type UpdateWorkflowVariableMutationVariables = Exact<{
  paramId: string;
  input: UpdateParameterInput;
}>;


export type UpdateWorkflowVariableMutation = { updateParameter: { id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config: any, createdAt: any, updatedAt: any } };

export type UpdateWorkflowVariablesMutationVariables = Exact<{
  input: ParameterBatchInput;
}>;


export type UpdateWorkflowVariablesMutation = { updateParameters: Array<{ id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config: any, createdAt: any, updatedAt: any }> };

export type DeleteWorkflowVariableMutationVariables = Exact<{
  input: RemoveParameterInput;
}>;


export type DeleteWorkflowVariableMutation = { removeParameter: boolean };

export type DeleteWorkflowVariablesMutationVariables = Exact<{
  input: RemoveParametersInput;
}>;


export type DeleteWorkflowVariablesMutation = { removeParameters: boolean };

export type CreateWorkspaceMutationVariables = Exact<{
  input: CreateWorkspaceInput;
}>;


export type CreateWorkspaceMutation = { createWorkspace: { workspace: { id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> } } | null };

export type GetWorkspacesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetWorkspacesQuery = { me: { id: string, workspaces: Array<{ id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> }> } | null };

export type GetWorkspaceByIdQueryVariables = Exact<{
  workspaceId: string;
}>;


export type GetWorkspaceByIdQuery = { node:
    | { __typename: 'Asset' }
    | { __typename: 'Deployment' }
    | { __typename: 'Job' }
    | { __typename: 'NodeExecution' }
    | { __typename: 'Project' }
    | { __typename: 'ProjectDocument' }
    | { __typename: 'Trigger' }
    | { __typename: 'User' }
    | { __typename: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> }
   | null };

export type UpdateWorkspaceMutationVariables = Exact<{
  input: UpdateWorkspaceInput;
}>;


export type UpdateWorkspaceMutation = { updateWorkspace: { workspace: { id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> } } | null };

export type DeleteWorkspaceMutationVariables = Exact<{
  input: DeleteWorkspaceInput;
}>;


export type DeleteWorkspaceMutation = { deleteWorkspace: { workspaceId: string } | null };

export type AddMemberToWorkspaceMutationVariables = Exact<{
  input: AddMemberToWorkspaceInput;
}>;


export type AddMemberToWorkspaceMutation = { addMemberToWorkspace: { workspace: { id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> } } | null };

export type RemoveMemberFromWorkspaceMutationVariables = Exact<{
  input: RemoveMemberFromWorkspaceInput;
}>;


export type RemoveMemberFromWorkspaceMutation = { removeMemberFromWorkspace: { workspace: { id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> } } | null };

export type UpdateMemberOfWorkspaceMutationVariables = Exact<{
  input: UpdateMemberOfWorkspaceInput;
}>;


export type UpdateMemberOfWorkspaceMutation = { updateMemberOfWorkspace: { workspace: { id: string, name: string, personal: boolean, members: Array<{ userId: string, role: Role, user: { id: string, email: string, name: string } | null }> } } | null };

export const DeploymentFragmentDoc = gql`
    fragment Deployment on Deployment {
  id
  projectId
  workspaceId
  workflowUrl
  description
  version
  createdAt
  updatedAt
  project {
    name
  }
}
    `;
export const ProjectFragmentDoc = gql`
    fragment Project on Project {
  id
  name
  description
  createdAt
  updatedAt
  workspaceId
  sharedToken
  deployment {
    ...Deployment
  }
  isLocked
}
    ${DeploymentFragmentDoc}`;
export const WorkspaceFragmentDoc = gql`
    fragment Workspace on Workspace {
  id
  name
  personal
  members {
    userId
    role
    user {
      id
      email
      name
    }
  }
}
    `;
export const ParameterFragmentDoc = gql`
    fragment Parameter on Parameter {
  id
  projectId
  index
  name
  defaultValue
  type
  required
  public
  config
  createdAt
  updatedAt
}
    `;
export const VariableFragmentDoc = gql`
    fragment Variable on Variable {
  key
  type
  value
}
    `;
export const TriggerFragmentDoc = gql`
    fragment Trigger on Trigger {
  id
  createdAt
  updatedAt
  lastTriggered
  workspaceId
  deploymentId
  deployment {
    ...Deployment
  }
  eventSource
  authToken
  timeInterval
  description
  variables {
    ...Variable
  }
  enabled
}
    ${DeploymentFragmentDoc}
${VariableFragmentDoc}`;
export const NodeExecutionFragmentDoc = gql`
    fragment NodeExecution on NodeExecution {
  id
  nodeId
  jobId
  status
  createdAt
  startedAt
  completedAt
}
    `;
export const JobFragmentDoc = gql`
    fragment Job on Job {
  id
  workspaceId
  status
  startedAt
  completedAt
  outputURLs
  userFacingLogsURL
  debug
  deployment {
    id
    description
  }
}
    `;
export const AssetFragmentDoc = gql`
    fragment Asset on Asset {
  id
  workspaceId
  createdAt
  fileName
  size
  contentType
  name
  url
  uuid
  flatFiles
  public
  archiveExtractionStatus
}
    `;
export const ProjectDocumentFragmentDoc = gql`
    fragment ProjectDocument on ProjectDocument {
  id
  timestamp
  updates
  version
}
    `;
export const ProjectSnapshotMetadataFragmentDoc = gql`
    fragment ProjectSnapshotMetadata on ProjectSnapshotMetadata {
  timestamp
  version
}
    `;
export const ProjectSnapshotFragmentDoc = gql`
    fragment ProjectSnapshot on ProjectSnapshot {
  timestamp
  updates
  version
}
    `;
export const UserFacingLogFragmentDoc = gql`
    fragment UserFacingLog on UserFacingLog {
  jobId
  timestamp
  nodeId
  nodeName
  level
  message
}
    `;
export const CmsProjectFragmentDoc = gql`
    fragment CmsProject on CMSProject {
  id
  name
  alias
  description
  license
  readme
  workspaceId
  visibility
  topics
  starCount
  createdAt
  updatedAt
}
    `;
export const CmsModelFragmentDoc = gql`
    fragment CmsModel on CMSModel {
  id
  projectId
  name
  description
  editorUrl
  key
  schema {
    schemaId
    fields {
      fieldId
      key
      type
      name
      description
    }
  }
  publicApiEp
  createdAt
  updatedAt
}
    `;
export const CmsItemFragmentDoc = gql`
    fragment CmsItem on CMSItem {
  id
  fields
  createdAt
  updatedAt
}
    `;
export const CmsAssetFragmentDoc = gql`
    fragment CmsAsset on CMSAsset {
  id
  uuid
  projectId
  filename
  size
  previewType
  url
  archiveExtractionStatus
  public
  createdAt
}
    `;
export const WorkerConfigFragmentDoc = gql`
    fragment WorkerConfig on WorkerConfig {
  id
  machineType
  computeCpuMilli
  computeMemoryMib
  bootDiskSizeGB
  taskCount
  maxConcurrency
  threadPoolSize
  channelBufferSize
  featureFlushThreshold
  nodeStatusPropagationDelayMilli
  createdAt
  updatedAt
}
    `;
export const GetAssetsDocument = gql`
    query GetAssets($workspaceId: ID!, $keyword: String, $pagination: PageBasedPagination!) {
  assets(workspaceId: $workspaceId, keyword: $keyword, pagination: $pagination) {
    totalCount
    nodes {
      ...Asset
    }
    pageInfo {
      totalCount
      currentPage
      totalPages
    }
  }
}
    ${AssetFragmentDoc}`;
export const CreateAssetDocument = gql`
    mutation CreateAsset($input: CreateAssetInput!) {
  createAsset(input: $input) {
    asset {
      ...Asset
    }
  }
}
    ${AssetFragmentDoc}`;
export const CreateAssetUploadDocument = gql`
    mutation CreateAssetUpload($input: CreateAssetUploadInput!) {
  createAssetUpload(input: $input) {
    token
    url
    contentType
    contentLength
    contentEncoding
    next
  }
}
    `;
export const UpdateAssetDocument = gql`
    mutation UpdateAsset($input: UpdateAssetInput!) {
  updateAsset(input: $input) {
    asset {
      ...Asset
    }
  }
}
    ${AssetFragmentDoc}`;
export const DeleteAssetDocument = gql`
    mutation DeleteAsset($input: DeleteAssetInput!) {
  deleteAsset(input: $input) {
    assetId
  }
}
    `;
export const GetCmsProjectByIdOrAliasDocument = gql`
    query GetCmsProjectByIdOrAlias($projectIdOrAlias: ID!) {
  cmsProject(projectIdOrAlias: $projectIdOrAlias) {
    ...CmsProject
  }
}
    ${CmsProjectFragmentDoc}`;
export const GetCmsProjectsDocument = gql`
    query GetCmsProjects($workspaceIds: [ID!]!, $keyword: String, $publicOnly: Boolean, $page: Int, $pageSize: Int) {
  cmsProjects(
    workspaceIds: $workspaceIds
    keyword: $keyword
    publicOnly: $publicOnly
    page: $page
    pageSize: $pageSize
  ) {
    ...CmsProject
  }
}
    ${CmsProjectFragmentDoc}`;
export const GetCmsModelsDocument = gql`
    query GetCmsModels($projectId: ID!, $page: Int, $pageSize: Int) {
  cmsModels(projectId: $projectId, page: $page, pageSize: $pageSize) {
    models {
      ...CmsModel
    }
    totalCount
  }
}
    ${CmsModelFragmentDoc}`;
export const GetCmsItemsDocument = gql`
    query GetCmsItems($projectId: ID!, $modelId: ID!, $keyword: String, $page: Int, $pageSize: Int) {
  cmsItems(
    projectId: $projectId
    modelId: $modelId
    keyword: $keyword
    page: $page
    pageSize: $pageSize
  ) {
    items {
      ...CmsItem
    }
    totalCount
  }
}
    ${CmsItemFragmentDoc}`;
export const GetCmsAssetDocument = gql`
    query GetCmsAsset($assetId: ID!) {
  cmsAsset(assetId: $assetId) {
    ...CmsAsset
  }
}
    ${CmsAssetFragmentDoc}`;
export const GetCmsModelExportUrlDocument = gql`
    query GetCmsModelExportUrl($projectId: ID!, $modelId: ID!, $exportType: CMSExportType!) {
  cmsModelExportUrl(
    projectId: $projectId
    modelId: $modelId
    exportType: $exportType
  )
}
    `;
export const CreateDeploymentDocument = gql`
    mutation CreateDeployment($input: CreateDeploymentInput!) {
  createDeployment(input: $input) {
    deployment {
      ...Deployment
    }
  }
}
    ${DeploymentFragmentDoc}`;
export const UpdateDeploymentDocument = gql`
    mutation UpdateDeployment($input: UpdateDeploymentInput!) {
  updateDeployment(input: $input) {
    deployment {
      ...Deployment
    }
  }
}
    ${DeploymentFragmentDoc}`;
export const DeleteDeploymentDocument = gql`
    mutation DeleteDeployment($input: DeleteDeploymentInput!) {
  deleteDeployment(input: $input) {
    deploymentId
  }
}
    `;
export const ExecuteDeploymentDocument = gql`
    mutation ExecuteDeployment($input: ExecuteDeploymentInput!) {
  executeDeployment(input: $input) {
    job {
      ...Job
    }
  }
}
    ${JobFragmentDoc}`;
export const GetDeploymentsDocument = gql`
    query GetDeployments($workspaceId: ID!, $keyword: String, $pagination: PageBasedPagination!) {
  deployments(
    workspaceId: $workspaceId
    keyword: $keyword
    pagination: $pagination
  ) {
    totalCount
    nodes {
      ...Deployment
    }
    pageInfo {
      totalCount
      currentPage
      totalPages
    }
  }
}
    ${DeploymentFragmentDoc}`;
export const GetDeploymentHeadDocument = gql`
    query GetDeploymentHead($input: GetHeadInput!) {
  deploymentHead(input: $input) {
    ...Deployment
  }
}
    ${DeploymentFragmentDoc}`;
export const GetLatestProjectSnapshotDocument = gql`
    query GetLatestProjectSnapshot($projectId: ID!) {
  latestProjectSnapshot(projectId: $projectId) {
    id
    timestamp
    updates
    version
  }
}
    `;
export const GetProjectSnapshotDocument = gql`
    query GetProjectSnapshot($projectId: ID!, $version: Int!) {
  projectSnapshot(projectId: $projectId, version: $version) {
    timestamp
    updates
    version
  }
}
    `;
export const GetProjectHistoryDocument = gql`
    query GetProjectHistory($projectId: ID!) {
  projectHistory(projectId: $projectId) {
    timestamp
    version
  }
}
    `;
export const PreviewSnapshotDocument = gql`
    mutation PreviewSnapshot($projectId: ID!, $version: Int!) {
  previewSnapshot(projectId: $projectId, version: $version) {
    id
    timestamp
    updates
    version
  }
}
    `;
export const RollbackProjectDocument = gql`
    mutation RollbackProject($projectId: ID!, $version: Int!) {
  rollbackProject(projectId: $projectId, version: $version) {
    id
    timestamp
    updates
    version
  }
}
    `;
export const SaveSnapshotDocument = gql`
    mutation SaveSnapshot($projectId: ID!) {
  saveSnapshot(projectId: $projectId)
}
    `;
export const GetJobsDocument = gql`
    query GetJobs($workspaceId: ID!, $keyword: String, $pagination: PageBasedPagination!) {
  jobs(workspaceId: $workspaceId, keyword: $keyword, pagination: $pagination) {
    totalCount
    nodes {
      ...Job
    }
    pageInfo {
      totalCount
      currentPage
      totalPages
    }
  }
}
    ${JobFragmentDoc}`;
export const GetJobDocument = gql`
    query GetJob($id: ID!) {
  job(id: $id) {
    ...Job
  }
}
    ${JobFragmentDoc}`;
export const GetNodeExecutionDocument = gql`
    query GetNodeExecution($jobId: ID!, $nodeId: String!) {
  nodeExecution(jobId: $jobId, nodeId: $nodeId) {
    ...NodeExecution
  }
}
    ${NodeExecutionFragmentDoc}`;
export const CancelJobDocument = gql`
    mutation CancelJob($input: CancelJobInput!) {
  cancelJob(input: $input) {
    job {
      ...Job
    }
  }
}
    ${JobFragmentDoc}`;
export const CreateProjectDocument = gql`
    mutation CreateProject($input: CreateProjectInput!) {
  createProject(input: $input) {
    project {
      ...Project
    }
  }
}
    ${ProjectFragmentDoc}`;
export const GetProjectsDocument = gql`
    query GetProjects($workspaceId: ID!, $keyword: String, $pagination: PageBasedPagination!) {
  projects(workspaceId: $workspaceId, keyword: $keyword, pagination: $pagination) {
    totalCount
    nodes {
      ...Project
    }
    pageInfo {
      totalCount
      currentPage
      totalPages
    }
  }
}
    ${ProjectFragmentDoc}`;
export const GetProjectByIdDocument = gql`
    query GetProjectById($projectId: ID!) {
  node(id: $projectId, type: PROJECT) {
    __typename
    ...Project
  }
}
    ${ProjectFragmentDoc}`;
export const UpdateProjectDocument = gql`
    mutation UpdateProject($input: UpdateProjectInput!) {
  updateProject(input: $input) {
    project {
      ...Project
    }
  }
}
    ${ProjectFragmentDoc}`;
export const DeleteProjectDocument = gql`
    mutation DeleteProject($input: DeleteProjectInput!) {
  deleteProject(input: $input) {
    projectId
  }
}
    `;
export const RunProjectDocument = gql`
    mutation RunProject($input: RunProjectInput!) {
  runProject(input: $input) {
    job {
      ...Job
    }
  }
}
    ${JobFragmentDoc}`;
export const CopyProjectDocument = gql`
    mutation CopyProject($projectId: ID!, $source: ID!) {
  copyProject(projectId: $projectId, source: $source)
}
    `;
export const ImportProjectDocument = gql`
    mutation ImportProject($projectId: ID!, $data: Bytes!) {
  importProject(projectId: $projectId, data: $data)
}
    `;
export const GetSharedProjectDocument = gql`
    query GetSharedProject($token: String!) {
  sharedProject(token: $token) {
    project {
      ...Project
    }
  }
}
    ${ProjectFragmentDoc}`;
export const GetSharedProjectInfoDocument = gql`
    query GetSharedProjectInfo($projectId: ID!) {
  projectSharingInfo(projectId: $projectId) {
    projectId
    sharingToken
  }
}
    `;
export const ShareProjectDocument = gql`
    mutation ShareProject($input: ShareProjectInput!) {
  shareProject(input: $input) {
    projectId
    sharingUrl
  }
}
    `;
export const UnshareProjectDocument = gql`
    mutation UnshareProject($input: UnshareProjectInput!) {
  unshareProject(input: $input) {
    projectId
  }
}
    `;
export const OnJobStatusChangeDocument = gql`
    subscription OnJobStatusChange($jobId: ID!) {
  jobStatus(jobId: $jobId)
}
    `;
export const OnNodeStatusChangeDocument = gql`
    subscription OnNodeStatusChange($jobId: ID!, $nodeId: String!) {
  nodeStatus(jobId: $jobId, nodeId: $nodeId)
}
    `;
export const UserFacingLogsDocument = gql`
    subscription UserFacingLogs($jobId: ID!) {
  userFacingLogs(jobId: $jobId) {
    jobId
    timestamp
    nodeId
    nodeName
    level
    message
  }
}
    `;
export const CreateTriggerDocument = gql`
    mutation CreateTrigger($input: CreateTriggerInput!) {
  createTrigger(input: $input) {
    ...Trigger
  }
}
    ${TriggerFragmentDoc}`;
export const UpdateTriggerDocument = gql`
    mutation UpdateTrigger($input: UpdateTriggerInput!) {
  updateTrigger(input: $input) {
    ...Trigger
  }
}
    ${TriggerFragmentDoc}`;
export const DeleteTriggerDocument = gql`
    mutation DeleteTrigger($triggerId: ID!) {
  deleteTrigger(triggerId: $triggerId)
}
    `;
export const GetTriggersDocument = gql`
    query GetTriggers($workspaceId: ID!, $keyword: String, $pagination: PageBasedPagination!) {
  triggers(workspaceId: $workspaceId, keyword: $keyword, pagination: $pagination) {
    totalCount
    nodes {
      ...Trigger
    }
    pageInfo {
      totalCount
      currentPage
      totalPages
    }
  }
}
    ${TriggerFragmentDoc}`;
export const GetMeDocument = gql`
    query GetMe {
  me {
    id
    name
    email
    myWorkspaceId
    lang
  }
}
    `;
export const GetMeAndWorkspacesDocument = gql`
    query GetMeAndWorkspaces {
  me {
    id
    name
    email
    myWorkspaceId
    lang
    workspaces {
      ...Workspace
    }
  }
}
    ${WorkspaceFragmentDoc}`;
export const SearchUserDocument = gql`
    query SearchUser($email: String!) {
  searchUser(nameOrEmail: $email) {
    id
    name
    email
  }
}
    `;
export const UpdateMeDocument = gql`
    mutation UpdateMe($input: UpdateMeInput!) {
  updateMe(input: $input) {
    me {
      id
      name
      email
      lang
    }
  }
}
    `;
export const GetWorkerConfigDocument = gql`
    query GetWorkerConfig {
  workerConfig {
    ...WorkerConfig
  }
}
    ${WorkerConfigFragmentDoc}`;
export const UpdateWorkerConfigDocument = gql`
    mutation UpdateWorkerConfig($input: UpdateWorkerConfigInput!) {
  updateWorkerConfig(input: $input) {
    config {
      ...WorkerConfig
    }
  }
}
    ${WorkerConfigFragmentDoc}`;
export const DeleteWorkerConfigDocument = gql`
    mutation DeleteWorkerConfig {
  deleteWorkerConfig {
    id
  }
}
    `;
export const GetWorkflowParametersDocument = gql`
    query GetWorkflowParameters($projectId: ID!) {
  parameters(projectId: $projectId) {
    ...Parameter
  }
}
    ${ParameterFragmentDoc}`;
export const CreateWorkflowVariableDocument = gql`
    mutation CreateWorkflowVariable($projectId: ID!, $input: DeclareParameterInput!) {
  declareParameter(projectId: $projectId, input: $input) {
    ...Parameter
  }
}
    ${ParameterFragmentDoc}`;
export const UpdateWorkflowVariableDocument = gql`
    mutation UpdateWorkflowVariable($paramId: ID!, $input: UpdateParameterInput!) {
  updateParameter(paramId: $paramId, input: $input) {
    ...Parameter
  }
}
    ${ParameterFragmentDoc}`;
export const UpdateWorkflowVariablesDocument = gql`
    mutation UpdateWorkflowVariables($input: ParameterBatchInput!) {
  updateParameters(input: $input) {
    ...Parameter
  }
}
    ${ParameterFragmentDoc}`;
export const DeleteWorkflowVariableDocument = gql`
    mutation DeleteWorkflowVariable($input: RemoveParameterInput!) {
  removeParameter(input: $input)
}
    `;
export const DeleteWorkflowVariablesDocument = gql`
    mutation DeleteWorkflowVariables($input: RemoveParametersInput!) {
  removeParameters(input: $input)
}
    `;
export const CreateWorkspaceDocument = gql`
    mutation CreateWorkspace($input: CreateWorkspaceInput!) {
  createWorkspace(input: $input) {
    workspace {
      ...Workspace
    }
  }
}
    ${WorkspaceFragmentDoc}`;
export const GetWorkspacesDocument = gql`
    query GetWorkspaces {
  me {
    id
    workspaces {
      ...Workspace
    }
  }
}
    ${WorkspaceFragmentDoc}`;
export const GetWorkspaceByIdDocument = gql`
    query GetWorkspaceById($workspaceId: ID!) {
  node(id: $workspaceId, type: WORKSPACE) {
    __typename
    ...Workspace
  }
}
    ${WorkspaceFragmentDoc}`;
export const UpdateWorkspaceDocument = gql`
    mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {
  updateWorkspace(input: $input) {
    workspace {
      ...Workspace
    }
  }
}
    ${WorkspaceFragmentDoc}`;
export const DeleteWorkspaceDocument = gql`
    mutation DeleteWorkspace($input: DeleteWorkspaceInput!) {
  deleteWorkspace(input: $input) {
    workspaceId
  }
}
    `;
export const AddMemberToWorkspaceDocument = gql`
    mutation AddMemberToWorkspace($input: AddMemberToWorkspaceInput!) {
  addMemberToWorkspace(input: $input) {
    workspace {
      ...Workspace
    }
  }
}
    ${WorkspaceFragmentDoc}`;
export const RemoveMemberFromWorkspaceDocument = gql`
    mutation RemoveMemberFromWorkspace($input: RemoveMemberFromWorkspaceInput!) {
  removeMemberFromWorkspace(input: $input) {
    workspace {
      ...Workspace
    }
  }
}
    ${WorkspaceFragmentDoc}`;
export const UpdateMemberOfWorkspaceDocument = gql`
    mutation UpdateMemberOfWorkspace($input: UpdateMemberOfWorkspaceInput!) {
  updateMemberOfWorkspace(input: $input) {
    workspace {
      ...Workspace
    }
  }
}
    ${WorkspaceFragmentDoc}`;

export type SdkFunctionWrapper = <T>(action: (requestHeaders?:Record<string, string>) => Promise<T>, operationName: string, operationType?: string, variables?: any) => Promise<T>;


const defaultWrapper: SdkFunctionWrapper = (action, _operationName, _operationType, _variables) => action();

export function getSdk(client: GraphQLClient, withWrapper: SdkFunctionWrapper = defaultWrapper) {
  return {
    GetAssets(variables: GetAssetsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetAssetsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetAssetsQuery>({ document: GetAssetsDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetAssets', 'query', variables);
    },
    CreateAsset(variables: CreateAssetMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CreateAssetMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateAssetMutation>({ document: CreateAssetDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CreateAsset', 'mutation', variables);
    },
    CreateAssetUpload(variables: CreateAssetUploadMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CreateAssetUploadMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateAssetUploadMutation>({ document: CreateAssetUploadDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CreateAssetUpload', 'mutation', variables);
    },
    UpdateAsset(variables: UpdateAssetMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateAssetMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateAssetMutation>({ document: UpdateAssetDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateAsset', 'mutation', variables);
    },
    DeleteAsset(variables: DeleteAssetMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteAssetMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteAssetMutation>({ document: DeleteAssetDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteAsset', 'mutation', variables);
    },
    GetCmsProjectByIdOrAlias(variables: GetCmsProjectByIdOrAliasQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetCmsProjectByIdOrAliasQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetCmsProjectByIdOrAliasQuery>({ document: GetCmsProjectByIdOrAliasDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetCmsProjectByIdOrAlias', 'query', variables);
    },
    GetCmsProjects(variables: GetCmsProjectsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetCmsProjectsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetCmsProjectsQuery>({ document: GetCmsProjectsDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetCmsProjects', 'query', variables);
    },
    GetCmsModels(variables: GetCmsModelsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetCmsModelsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetCmsModelsQuery>({ document: GetCmsModelsDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetCmsModels', 'query', variables);
    },
    GetCmsItems(variables: GetCmsItemsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetCmsItemsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetCmsItemsQuery>({ document: GetCmsItemsDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetCmsItems', 'query', variables);
    },
    GetCmsAsset(variables: GetCmsAssetQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetCmsAssetQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetCmsAssetQuery>({ document: GetCmsAssetDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetCmsAsset', 'query', variables);
    },
    GetCmsModelExportUrl(variables: GetCmsModelExportUrlQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetCmsModelExportUrlQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetCmsModelExportUrlQuery>({ document: GetCmsModelExportUrlDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetCmsModelExportUrl', 'query', variables);
    },
    CreateDeployment(variables: CreateDeploymentMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CreateDeploymentMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateDeploymentMutation>({ document: CreateDeploymentDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CreateDeployment', 'mutation', variables);
    },
    UpdateDeployment(variables: UpdateDeploymentMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateDeploymentMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateDeploymentMutation>({ document: UpdateDeploymentDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateDeployment', 'mutation', variables);
    },
    DeleteDeployment(variables: DeleteDeploymentMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteDeploymentMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteDeploymentMutation>({ document: DeleteDeploymentDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteDeployment', 'mutation', variables);
    },
    ExecuteDeployment(variables: ExecuteDeploymentMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<ExecuteDeploymentMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<ExecuteDeploymentMutation>({ document: ExecuteDeploymentDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'ExecuteDeployment', 'mutation', variables);
    },
    GetDeployments(variables: GetDeploymentsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetDeploymentsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetDeploymentsQuery>({ document: GetDeploymentsDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetDeployments', 'query', variables);
    },
    GetDeploymentHead(variables: GetDeploymentHeadQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetDeploymentHeadQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetDeploymentHeadQuery>({ document: GetDeploymentHeadDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetDeploymentHead', 'query', variables);
    },
    GetLatestProjectSnapshot(variables: GetLatestProjectSnapshotQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetLatestProjectSnapshotQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetLatestProjectSnapshotQuery>({ document: GetLatestProjectSnapshotDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetLatestProjectSnapshot', 'query', variables);
    },
    GetProjectSnapshot(variables: GetProjectSnapshotQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetProjectSnapshotQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetProjectSnapshotQuery>({ document: GetProjectSnapshotDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetProjectSnapshot', 'query', variables);
    },
    GetProjectHistory(variables: GetProjectHistoryQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetProjectHistoryQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetProjectHistoryQuery>({ document: GetProjectHistoryDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetProjectHistory', 'query', variables);
    },
    PreviewSnapshot(variables: PreviewSnapshotMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<PreviewSnapshotMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<PreviewSnapshotMutation>({ document: PreviewSnapshotDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'PreviewSnapshot', 'mutation', variables);
    },
    RollbackProject(variables: RollbackProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<RollbackProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<RollbackProjectMutation>({ document: RollbackProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'RollbackProject', 'mutation', variables);
    },
    SaveSnapshot(variables: SaveSnapshotMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<SaveSnapshotMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<SaveSnapshotMutation>({ document: SaveSnapshotDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'SaveSnapshot', 'mutation', variables);
    },
    GetJobs(variables: GetJobsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetJobsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetJobsQuery>({ document: GetJobsDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetJobs', 'query', variables);
    },
    GetJob(variables: GetJobQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetJobQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetJobQuery>({ document: GetJobDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetJob', 'query', variables);
    },
    GetNodeExecution(variables: GetNodeExecutionQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetNodeExecutionQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetNodeExecutionQuery>({ document: GetNodeExecutionDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetNodeExecution', 'query', variables);
    },
    CancelJob(variables: CancelJobMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CancelJobMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CancelJobMutation>({ document: CancelJobDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CancelJob', 'mutation', variables);
    },
    CreateProject(variables: CreateProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CreateProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateProjectMutation>({ document: CreateProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CreateProject', 'mutation', variables);
    },
    GetProjects(variables: GetProjectsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetProjectsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetProjectsQuery>({ document: GetProjectsDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetProjects', 'query', variables);
    },
    GetProjectById(variables: GetProjectByIdQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetProjectByIdQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetProjectByIdQuery>({ document: GetProjectByIdDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetProjectById', 'query', variables);
    },
    UpdateProject(variables: UpdateProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateProjectMutation>({ document: UpdateProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateProject', 'mutation', variables);
    },
    DeleteProject(variables: DeleteProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteProjectMutation>({ document: DeleteProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteProject', 'mutation', variables);
    },
    RunProject(variables: RunProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<RunProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<RunProjectMutation>({ document: RunProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'RunProject', 'mutation', variables);
    },
    CopyProject(variables: CopyProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CopyProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CopyProjectMutation>({ document: CopyProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CopyProject', 'mutation', variables);
    },
    ImportProject(variables: ImportProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<ImportProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<ImportProjectMutation>({ document: ImportProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'ImportProject', 'mutation', variables);
    },
    GetSharedProject(variables: GetSharedProjectQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetSharedProjectQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetSharedProjectQuery>({ document: GetSharedProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetSharedProject', 'query', variables);
    },
    GetSharedProjectInfo(variables: GetSharedProjectInfoQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetSharedProjectInfoQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetSharedProjectInfoQuery>({ document: GetSharedProjectInfoDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetSharedProjectInfo', 'query', variables);
    },
    ShareProject(variables: ShareProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<ShareProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<ShareProjectMutation>({ document: ShareProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'ShareProject', 'mutation', variables);
    },
    UnshareProject(variables: UnshareProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UnshareProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UnshareProjectMutation>({ document: UnshareProjectDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UnshareProject', 'mutation', variables);
    },
    OnJobStatusChange(variables: OnJobStatusChangeSubscriptionVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<OnJobStatusChangeSubscription> {
      return withWrapper((wrappedRequestHeaders) => client.request<OnJobStatusChangeSubscription>({ document: OnJobStatusChangeDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'OnJobStatusChange', 'subscription', variables);
    },
    OnNodeStatusChange(variables: OnNodeStatusChangeSubscriptionVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<OnNodeStatusChangeSubscription> {
      return withWrapper((wrappedRequestHeaders) => client.request<OnNodeStatusChangeSubscription>({ document: OnNodeStatusChangeDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'OnNodeStatusChange', 'subscription', variables);
    },
    UserFacingLogs(variables: UserFacingLogsSubscriptionVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UserFacingLogsSubscription> {
      return withWrapper((wrappedRequestHeaders) => client.request<UserFacingLogsSubscription>({ document: UserFacingLogsDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UserFacingLogs', 'subscription', variables);
    },
    CreateTrigger(variables: CreateTriggerMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CreateTriggerMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateTriggerMutation>({ document: CreateTriggerDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CreateTrigger', 'mutation', variables);
    },
    UpdateTrigger(variables: UpdateTriggerMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateTriggerMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateTriggerMutation>({ document: UpdateTriggerDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateTrigger', 'mutation', variables);
    },
    DeleteTrigger(variables: DeleteTriggerMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteTriggerMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteTriggerMutation>({ document: DeleteTriggerDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteTrigger', 'mutation', variables);
    },
    GetTriggers(variables: GetTriggersQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetTriggersQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetTriggersQuery>({ document: GetTriggersDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetTriggers', 'query', variables);
    },
    GetMe(variables?: GetMeQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetMeQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetMeQuery>({ document: GetMeDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetMe', 'query', variables);
    },
    GetMeAndWorkspaces(variables?: GetMeAndWorkspacesQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetMeAndWorkspacesQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetMeAndWorkspacesQuery>({ document: GetMeAndWorkspacesDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetMeAndWorkspaces', 'query', variables);
    },
    SearchUser(variables: SearchUserQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<SearchUserQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<SearchUserQuery>({ document: SearchUserDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'SearchUser', 'query', variables);
    },
    UpdateMe(variables: UpdateMeMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateMeMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateMeMutation>({ document: UpdateMeDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateMe', 'mutation', variables);
    },
    GetWorkerConfig(variables?: GetWorkerConfigQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetWorkerConfigQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetWorkerConfigQuery>({ document: GetWorkerConfigDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetWorkerConfig', 'query', variables);
    },
    UpdateWorkerConfig(variables: UpdateWorkerConfigMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateWorkerConfigMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateWorkerConfigMutation>({ document: UpdateWorkerConfigDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateWorkerConfig', 'mutation', variables);
    },
    DeleteWorkerConfig(variables?: DeleteWorkerConfigMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteWorkerConfigMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteWorkerConfigMutation>({ document: DeleteWorkerConfigDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteWorkerConfig', 'mutation', variables);
    },
    GetWorkflowParameters(variables: GetWorkflowParametersQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetWorkflowParametersQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetWorkflowParametersQuery>({ document: GetWorkflowParametersDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetWorkflowParameters', 'query', variables);
    },
    CreateWorkflowVariable(variables: CreateWorkflowVariableMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CreateWorkflowVariableMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateWorkflowVariableMutation>({ document: CreateWorkflowVariableDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CreateWorkflowVariable', 'mutation', variables);
    },
    UpdateWorkflowVariable(variables: UpdateWorkflowVariableMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateWorkflowVariableMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateWorkflowVariableMutation>({ document: UpdateWorkflowVariableDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateWorkflowVariable', 'mutation', variables);
    },
    UpdateWorkflowVariables(variables: UpdateWorkflowVariablesMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateWorkflowVariablesMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateWorkflowVariablesMutation>({ document: UpdateWorkflowVariablesDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateWorkflowVariables', 'mutation', variables);
    },
    DeleteWorkflowVariable(variables: DeleteWorkflowVariableMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteWorkflowVariableMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteWorkflowVariableMutation>({ document: DeleteWorkflowVariableDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteWorkflowVariable', 'mutation', variables);
    },
    DeleteWorkflowVariables(variables: DeleteWorkflowVariablesMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteWorkflowVariablesMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteWorkflowVariablesMutation>({ document: DeleteWorkflowVariablesDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteWorkflowVariables', 'mutation', variables);
    },
    CreateWorkspace(variables: CreateWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CreateWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateWorkspaceMutation>({ document: CreateWorkspaceDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CreateWorkspace', 'mutation', variables);
    },
    GetWorkspaces(variables?: GetWorkspacesQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetWorkspacesQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetWorkspacesQuery>({ document: GetWorkspacesDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetWorkspaces', 'query', variables);
    },
    GetWorkspaceById(variables: GetWorkspaceByIdQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetWorkspaceByIdQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetWorkspaceByIdQuery>({ document: GetWorkspaceByIdDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetWorkspaceById', 'query', variables);
    },
    UpdateWorkspace(variables: UpdateWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateWorkspaceMutation>({ document: UpdateWorkspaceDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateWorkspace', 'mutation', variables);
    },
    DeleteWorkspace(variables: DeleteWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteWorkspaceMutation>({ document: DeleteWorkspaceDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteWorkspace', 'mutation', variables);
    },
    AddMemberToWorkspace(variables: AddMemberToWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<AddMemberToWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<AddMemberToWorkspaceMutation>({ document: AddMemberToWorkspaceDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'AddMemberToWorkspace', 'mutation', variables);
    },
    RemoveMemberFromWorkspace(variables: RemoveMemberFromWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<RemoveMemberFromWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<RemoveMemberFromWorkspaceMutation>({ document: RemoveMemberFromWorkspaceDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'RemoveMemberFromWorkspace', 'mutation', variables);
    },
    UpdateMemberOfWorkspace(variables: UpdateMemberOfWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateMemberOfWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateMemberOfWorkspaceMutation>({ document: UpdateMemberOfWorkspaceDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateMemberOfWorkspace', 'mutation', variables);
    }
  };
}
export type Sdk = ReturnType<typeof getSdk>;