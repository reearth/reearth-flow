import { GraphQLClient, RequestOptions } from 'graphql-request';
import gql from 'graphql-tag';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
type GraphQLClientRequestHeaders = RequestOptions['requestHeaders'];
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  Any: { input: any; output: any; }
  Bytes: { input: any; output: any; }
  DateTime: { input: any; output: any; }
  FileSize: { input: any; output: any; }
  JSON: { input: any; output: any; }
  Lang: { input: any; output: any; }
  URL: { input: any; output: any; }
  Upload: { input: any; output: any; }
};

export type ApiDriverInput = {
  token: Scalars['String']['input'];
};

export type AddMemberToWorkspaceInput = {
  role: Role;
  userId: Scalars['ID']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type AddMemberToWorkspacePayload = {
  __typename?: 'AddMemberToWorkspacePayload';
  workspace: Workspace;
};

export enum ArchiveExtractionStatus {
  Done = 'DONE',
  Failed = 'FAILED',
  InProgress = 'IN_PROGRESS',
  Pending = 'PENDING',
  Skipped = 'SKIPPED'
}

export type Asset = Node & {
  __typename?: 'Asset';
  Workspace?: Maybe<Workspace>;
  archiveExtractionStatus?: Maybe<ArchiveExtractionStatus>;
  contentType: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  fileName: Scalars['String']['output'];
  flatFiles: Scalars['Boolean']['output'];
  id: Scalars['ID']['output'];
  name: Scalars['String']['output'];
  public: Scalars['Boolean']['output'];
  size: Scalars['FileSize']['output'];
  url: Scalars['String']['output'];
  uuid: Scalars['String']['output'];
  workspaceId: Scalars['ID']['output'];
};

export type AssetConnection = {
  __typename?: 'AssetConnection';
  nodes: Array<Maybe<Asset>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export enum AssetSortType {
  Date = 'DATE',
  Name = 'NAME',
  Size = 'SIZE'
}

export type CmsAsset = {
  __typename?: 'CMSAsset';
  archiveExtractionStatus?: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['DateTime']['output'];
  filename: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  previewType?: Maybe<Scalars['String']['output']>;
  projectId: Scalars['ID']['output'];
  public: Scalars['Boolean']['output'];
  size: Scalars['Int']['output'];
  url: Scalars['String']['output'];
  uuid: Scalars['String']['output'];
};

export type CmsAssetsConnection = {
  __typename?: 'CMSAssetsConnection';
  assets: Array<CmsAsset>;
  pageInfo: CmsPageInfo;
  totalCount: Scalars['Int']['output'];
};

export enum CmsExportType {
  Geojson = 'GEOJSON',
  Json = 'JSON'
}

export type CmsItem = {
  __typename?: 'CMSItem';
  createdAt: Scalars['DateTime']['output'];
  fields: Scalars['JSON']['output'];
  id: Scalars['ID']['output'];
  updatedAt: Scalars['DateTime']['output'];
};

export type CmsItemsConnection = {
  __typename?: 'CMSItemsConnection';
  items: Array<CmsItem>;
  totalCount: Scalars['Int']['output'];
};

export type CmsModel = {
  __typename?: 'CMSModel';
  createdAt: Scalars['DateTime']['output'];
  description: Scalars['String']['output'];
  editorUrl: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  key: Scalars['String']['output'];
  name: Scalars['String']['output'];
  projectId: Scalars['ID']['output'];
  publicApiEp: Scalars['String']['output'];
  schema: CmsSchema;
  updatedAt: Scalars['DateTime']['output'];
};

export type CmsModelsConnection = {
  __typename?: 'CMSModelsConnection';
  models: Array<CmsModel>;
  pageInfo: CmsPageInfo;
  totalCount: Scalars['Int']['output'];
};

export type CmsPageInfo = {
  __typename?: 'CMSPageInfo';
  page: Scalars['Int']['output'];
  pageSize: Scalars['Int']['output'];
};

export type CmsProject = {
  __typename?: 'CMSProject';
  alias: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  description?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  license?: Maybe<Scalars['String']['output']>;
  name: Scalars['String']['output'];
  readme?: Maybe<Scalars['String']['output']>;
  starCount: Scalars['Int']['output'];
  topics: Array<Scalars['String']['output']>;
  updatedAt: Scalars['DateTime']['output'];
  visibility: CmsVisibility;
  workspaceId: Scalars['ID']['output'];
};

export type CmsSchema = {
  __typename?: 'CMSSchema';
  fields: Array<CmsSchemaField>;
  schemaId: Scalars['ID']['output'];
};

export type CmsSchemaField = {
  __typename?: 'CMSSchemaField';
  description?: Maybe<Scalars['String']['output']>;
  fieldId: Scalars['ID']['output'];
  key: Scalars['String']['output'];
  name: Scalars['String']['output'];
  type: CmsSchemaFieldType;
};

export enum CmsSchemaFieldType {
  Asset = 'ASSET',
  Bool = 'BOOL',
  Checkbox = 'CHECKBOX',
  Date = 'DATE',
  Geometryeditor = 'GEOMETRYEDITOR',
  Geometryobject = 'GEOMETRYOBJECT',
  Group = 'GROUP',
  Integer = 'INTEGER',
  Markdowntext = 'MARKDOWNTEXT',
  Number = 'NUMBER',
  Reference = 'REFERENCE',
  Richtext = 'RICHTEXT',
  Select = 'SELECT',
  Tag = 'TAG',
  Text = 'TEXT',
  Textarea = 'TEXTAREA',
  Url = 'URL'
}

export enum CmsVisibility {
  Private = 'PRIVATE',
  Public = 'PUBLIC'
}

export type CancelJobInput = {
  jobId: Scalars['ID']['input'];
};

export type CancelJobPayload = {
  __typename?: 'CancelJobPayload';
  job?: Maybe<Job>;
};

export type CreateAssetInput = {
  file?: InputMaybe<Scalars['Upload']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
  token?: InputMaybe<Scalars['String']['input']>;
  workspaceId: Scalars['ID']['input'];
};

export type CreateAssetPayload = {
  __typename?: 'CreateAssetPayload';
  asset: Asset;
};

export type CreateAssetUploadInput = {
  contentEncoding?: InputMaybe<Scalars['String']['input']>;
  contentLength?: InputMaybe<Scalars['Int']['input']>;
  cursor?: InputMaybe<Scalars['String']['input']>;
  filename?: InputMaybe<Scalars['String']['input']>;
  workspaceId: Scalars['ID']['input'];
};

export type CreateAssetUploadPayload = {
  __typename?: 'CreateAssetUploadPayload';
  contentEncoding?: Maybe<Scalars['String']['output']>;
  contentLength: Scalars['Int']['output'];
  contentType?: Maybe<Scalars['String']['output']>;
  next?: Maybe<Scalars['String']['output']>;
  token: Scalars['String']['output'];
  url: Scalars['String']['output'];
};

export type CreateDeploymentInput = {
  description: Scalars['String']['input'];
  file: Scalars['Upload']['input'];
  projectId?: InputMaybe<Scalars['ID']['input']>;
  workspaceId: Scalars['ID']['input'];
};

export type CreateProjectInput = {
  archived?: InputMaybe<Scalars['Boolean']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
  workspaceId: Scalars['ID']['input'];
};

export type CreateTriggerInput = {
  apiDriverInput?: InputMaybe<ApiDriverInput>;
  deploymentId: Scalars['ID']['input'];
  description: Scalars['String']['input'];
  timeDriverInput?: InputMaybe<TimeDriverInput>;
  workspaceId: Scalars['ID']['input'];
};

export type CreateWorkspaceInput = {
  name: Scalars['String']['input'];
};

export type CreateWorkspacePayload = {
  __typename?: 'CreateWorkspacePayload';
  workspace: Workspace;
};

export type DeclareParameterInput = {
  config?: InputMaybe<Scalars['JSON']['input']>;
  defaultValue?: InputMaybe<Scalars['Any']['input']>;
  index?: InputMaybe<Scalars['Int']['input']>;
  name: Scalars['String']['input'];
  public: Scalars['Boolean']['input'];
  required: Scalars['Boolean']['input'];
  type: ParameterType;
};

export type DeleteAssetInput = {
  assetId: Scalars['ID']['input'];
};

export type DeleteAssetPayload = {
  __typename?: 'DeleteAssetPayload';
  assetId: Scalars['ID']['output'];
};

export type DeleteDeploymentInput = {
  deploymentId: Scalars['ID']['input'];
};

export type DeleteDeploymentPayload = {
  __typename?: 'DeleteDeploymentPayload';
  deploymentId: Scalars['ID']['output'];
};

export type DeleteMeInput = {
  userId: Scalars['ID']['input'];
};

export type DeleteMePayload = {
  __typename?: 'DeleteMePayload';
  userId: Scalars['ID']['output'];
};

export type DeleteProjectInput = {
  projectId: Scalars['ID']['input'];
};

export type DeleteProjectPayload = {
  __typename?: 'DeleteProjectPayload';
  projectId: Scalars['ID']['output'];
};

export type DeleteWorkspaceInput = {
  workspaceId: Scalars['ID']['input'];
};

export type DeleteWorkspacePayload = {
  __typename?: 'DeleteWorkspacePayload';
  workspaceId: Scalars['ID']['output'];
};

export type Deployment = Node & {
  __typename?: 'Deployment';
  createdAt: Scalars['DateTime']['output'];
  description: Scalars['String']['output'];
  headId?: Maybe<Scalars['ID']['output']>;
  id: Scalars['ID']['output'];
  isHead: Scalars['Boolean']['output'];
  project?: Maybe<Project>;
  projectId?: Maybe<Scalars['ID']['output']>;
  updatedAt: Scalars['DateTime']['output'];
  version: Scalars['String']['output'];
  workflowUrl: Scalars['String']['output'];
  workspace?: Maybe<Workspace>;
  workspaceId: Scalars['ID']['output'];
};

export type DeploymentConnection = {
  __typename?: 'DeploymentConnection';
  nodes: Array<Maybe<Deployment>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export type DeploymentPayload = {
  __typename?: 'DeploymentPayload';
  deployment: Deployment;
};

export enum DeploymentSortField {
  Description = 'DESCRIPTION',
  UpdatedAt = 'UPDATED_AT',
  Version = 'VERSION'
}

export enum EventSourceType {
  ApiDriven = 'API_DRIVEN',
  TimeDriven = 'TIME_DRIVEN'
}

export type ExecuteDeploymentInput = {
  deploymentId: Scalars['ID']['input'];
};

export type GetByVersionInput = {
  projectId?: InputMaybe<Scalars['ID']['input']>;
  version: Scalars['String']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type GetHeadInput = {
  projectId?: InputMaybe<Scalars['ID']['input']>;
  workspaceId: Scalars['ID']['input'];
};

export type Job = Node & {
  __typename?: 'Job';
  completedAt?: Maybe<Scalars['DateTime']['output']>;
  debug?: Maybe<Scalars['Boolean']['output']>;
  deployment?: Maybe<Deployment>;
  deploymentId: Scalars['ID']['output'];
  id: Scalars['ID']['output'];
  logs?: Maybe<Array<Maybe<Log>>>;
  logsURL?: Maybe<Scalars['String']['output']>;
  outputURLs?: Maybe<Array<Scalars['String']['output']>>;
  startedAt: Scalars['DateTime']['output'];
  status: JobStatus;
  userFacingLogsURL?: Maybe<Scalars['String']['output']>;
  workerLogsURL?: Maybe<Scalars['String']['output']>;
  workspace?: Maybe<Workspace>;
  workspaceId: Scalars['ID']['output'];
};


export type JobLogsArgs = {
  since: Scalars['DateTime']['input'];
};

export type JobConnection = {
  __typename?: 'JobConnection';
  nodes: Array<Maybe<Job>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export type JobPayload = {
  __typename?: 'JobPayload';
  job: Job;
};

export enum JobSortField {
  CompletedAt = 'COMPLETED_AT',
  StartedAt = 'STARTED_AT',
  Status = 'STATUS'
}

export enum JobStatus {
  Cancelled = 'CANCELLED',
  Completed = 'COMPLETED',
  Failed = 'FAILED',
  Pending = 'PENDING',
  Running = 'RUNNING'
}

export type Log = {
  __typename?: 'Log';
  jobId: Scalars['ID']['output'];
  logLevel: LogLevel;
  message: Scalars['String']['output'];
  nodeId?: Maybe<Scalars['ID']['output']>;
  timestamp: Scalars['DateTime']['output'];
};

export enum LogLevel {
  Debug = 'DEBUG',
  Error = 'ERROR',
  Info = 'INFO',
  Trace = 'TRACE',
  Warn = 'WARN'
}

export type Me = {
  __typename?: 'Me';
  auths: Array<Scalars['String']['output']>;
  email: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  lang: Scalars['Lang']['output'];
  myWorkspace?: Maybe<Workspace>;
  myWorkspaceId: Scalars['ID']['output'];
  name: Scalars['String']['output'];
  workspaces: Array<Workspace>;
};

export type Mutation = {
  __typename?: 'Mutation';
  addMemberToWorkspace?: Maybe<AddMemberToWorkspacePayload>;
  cancelJob: CancelJobPayload;
  copyProject: Scalars['Boolean']['output'];
  createAsset?: Maybe<CreateAssetPayload>;
  createAssetUpload?: Maybe<CreateAssetUploadPayload>;
  createDeployment?: Maybe<DeploymentPayload>;
  createProject?: Maybe<ProjectPayload>;
  createTrigger: Trigger;
  createWorkspace?: Maybe<CreateWorkspacePayload>;
  declareParameter: Parameter;
  deleteAsset?: Maybe<DeleteAssetPayload>;
  deleteDeployment?: Maybe<DeleteDeploymentPayload>;
  deleteMe?: Maybe<DeleteMePayload>;
  deleteProject?: Maybe<DeleteProjectPayload>;
  deleteTrigger: Scalars['Boolean']['output'];
  deleteWorkspace?: Maybe<DeleteWorkspacePayload>;
  executeDeployment?: Maybe<JobPayload>;
  importProject: Scalars['Boolean']['output'];
  previewSnapshot?: Maybe<PreviewSnapshot>;
  removeMemberFromWorkspace?: Maybe<RemoveMemberFromWorkspacePayload>;
  removeMyAuth?: Maybe<UpdateMePayload>;
  removeParameter: Scalars['Boolean']['output'];
  removeParameters: Scalars['Boolean']['output'];
  rollbackProject?: Maybe<ProjectDocument>;
  runProject?: Maybe<RunProjectPayload>;
  saveSnapshot: Scalars['Boolean']['output'];
  shareProject?: Maybe<ShareProjectPayload>;
  signup?: Maybe<SignupPayload>;
  unshareProject?: Maybe<UnshareProjectPayload>;
  updateAsset?: Maybe<UpdateAssetPayload>;
  updateDeployment?: Maybe<DeploymentPayload>;
  updateMe?: Maybe<UpdateMePayload>;
  updateMemberOfWorkspace?: Maybe<UpdateMemberOfWorkspacePayload>;
  updateParameter: Parameter;
  updateParameterOrder: Array<Parameter>;
  updateParameters: Array<Parameter>;
  updateProject?: Maybe<ProjectPayload>;
  updateTrigger: Trigger;
  updateWorkspace?: Maybe<UpdateWorkspacePayload>;
};


export type MutationAddMemberToWorkspaceArgs = {
  input: AddMemberToWorkspaceInput;
};


export type MutationCancelJobArgs = {
  input: CancelJobInput;
};


export type MutationCopyProjectArgs = {
  projectId: Scalars['ID']['input'];
  source: Scalars['ID']['input'];
};


export type MutationCreateAssetArgs = {
  input: CreateAssetInput;
};


export type MutationCreateAssetUploadArgs = {
  input: CreateAssetUploadInput;
};


export type MutationCreateDeploymentArgs = {
  input: CreateDeploymentInput;
};


export type MutationCreateProjectArgs = {
  input: CreateProjectInput;
};


export type MutationCreateTriggerArgs = {
  input: CreateTriggerInput;
};


export type MutationCreateWorkspaceArgs = {
  input: CreateWorkspaceInput;
};


export type MutationDeclareParameterArgs = {
  input: DeclareParameterInput;
  projectId: Scalars['ID']['input'];
};


export type MutationDeleteAssetArgs = {
  input: DeleteAssetInput;
};


export type MutationDeleteDeploymentArgs = {
  input: DeleteDeploymentInput;
};


export type MutationDeleteMeArgs = {
  input: DeleteMeInput;
};


export type MutationDeleteProjectArgs = {
  input: DeleteProjectInput;
};


export type MutationDeleteTriggerArgs = {
  triggerId: Scalars['ID']['input'];
};


export type MutationDeleteWorkspaceArgs = {
  input: DeleteWorkspaceInput;
};


export type MutationExecuteDeploymentArgs = {
  input: ExecuteDeploymentInput;
};


export type MutationImportProjectArgs = {
  data: Scalars['Bytes']['input'];
  projectId: Scalars['ID']['input'];
};


export type MutationPreviewSnapshotArgs = {
  name?: InputMaybe<Scalars['String']['input']>;
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
};


export type MutationRemoveMemberFromWorkspaceArgs = {
  input: RemoveMemberFromWorkspaceInput;
};


export type MutationRemoveMyAuthArgs = {
  input: RemoveMyAuthInput;
};


export type MutationRemoveParameterArgs = {
  input: RemoveParameterInput;
};


export type MutationRemoveParametersArgs = {
  input: RemoveParametersInput;
};


export type MutationRollbackProjectArgs = {
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
};


export type MutationRunProjectArgs = {
  input: RunProjectInput;
};


export type MutationSaveSnapshotArgs = {
  projectId: Scalars['ID']['input'];
};


export type MutationShareProjectArgs = {
  input: ShareProjectInput;
};


export type MutationSignupArgs = {
  input: SignupInput;
};


export type MutationUnshareProjectArgs = {
  input: UnshareProjectInput;
};


export type MutationUpdateAssetArgs = {
  input: UpdateAssetInput;
};


export type MutationUpdateDeploymentArgs = {
  input: UpdateDeploymentInput;
};


export type MutationUpdateMeArgs = {
  input: UpdateMeInput;
};


export type MutationUpdateMemberOfWorkspaceArgs = {
  input: UpdateMemberOfWorkspaceInput;
};


export type MutationUpdateParameterArgs = {
  input: UpdateParameterInput;
  paramId: Scalars['ID']['input'];
};


export type MutationUpdateParameterOrderArgs = {
  input: UpdateParameterOrderInput;
  projectId: Scalars['ID']['input'];
};


export type MutationUpdateParametersArgs = {
  input: ParameterBatchInput;
};


export type MutationUpdateProjectArgs = {
  input: UpdateProjectInput;
};


export type MutationUpdateTriggerArgs = {
  input: UpdateTriggerInput;
};


export type MutationUpdateWorkspaceArgs = {
  input: UpdateWorkspaceInput;
};

export type Node = {
  id: Scalars['ID']['output'];
};

export type NodeExecution = Node & {
  __typename?: 'NodeExecution';
  completedAt?: Maybe<Scalars['DateTime']['output']>;
  createdAt?: Maybe<Scalars['DateTime']['output']>;
  id: Scalars['ID']['output'];
  jobId: Scalars['ID']['output'];
  nodeId: Scalars['ID']['output'];
  startedAt?: Maybe<Scalars['DateTime']['output']>;
  status: NodeStatus;
};

export enum NodeStatus {
  Completed = 'COMPLETED',
  Failed = 'FAILED',
  Pending = 'PENDING',
  Processing = 'PROCESSING',
  Starting = 'STARTING'
}

export enum NodeType {
  Asset = 'ASSET',
  Project = 'PROJECT',
  User = 'USER',
  Workspace = 'WORKSPACE'
}

export enum OrderDirection {
  Asc = 'ASC',
  Desc = 'DESC'
}

export type PageBasedPagination = {
  orderBy?: InputMaybe<Scalars['String']['input']>;
  orderDir?: InputMaybe<OrderDirection>;
  page: Scalars['Int']['input'];
  pageSize: Scalars['Int']['input'];
};

export type PageInfo = {
  __typename?: 'PageInfo';
  currentPage?: Maybe<Scalars['Int']['output']>;
  totalCount: Scalars['Int']['output'];
  totalPages?: Maybe<Scalars['Int']['output']>;
};

export type Pagination = {
  orderBy?: InputMaybe<Scalars['String']['input']>;
  orderDir?: InputMaybe<OrderDirection>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
};

export type Parameter = {
  __typename?: 'Parameter';
  config?: Maybe<Scalars['JSON']['output']>;
  createdAt: Scalars['DateTime']['output'];
  defaultValue: Scalars['Any']['output'];
  id: Scalars['ID']['output'];
  index: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  projectId: Scalars['ID']['output'];
  public: Scalars['Boolean']['output'];
  required: Scalars['Boolean']['output'];
  type: ParameterType;
  updatedAt: Scalars['DateTime']['output'];
};

export type ParameterBatchInput = {
  creates?: InputMaybe<Array<DeclareParameterInput>>;
  deletes?: InputMaybe<Array<Scalars['ID']['input']>>;
  projectId: Scalars['ID']['input'];
  reorders?: InputMaybe<Array<UpdateParameterOrderInput>>;
  updates?: InputMaybe<Array<ParameterUpdateItem>>;
};

export enum ParameterType {
  Array = 'ARRAY',
  Choice = 'CHOICE',
  Color = 'COLOR',
  Datetime = 'DATETIME',
  FileFolder = 'FILE_FOLDER',
  Number = 'NUMBER',
  Text = 'TEXT',
  YesNo = 'YES_NO'
}

export type ParameterUpdateItem = {
  config?: InputMaybe<Scalars['JSON']['input']>;
  defaultValue?: InputMaybe<Scalars['Any']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
  paramId: Scalars['ID']['input'];
  public?: InputMaybe<Scalars['Boolean']['input']>;
  required?: InputMaybe<Scalars['Boolean']['input']>;
  type?: InputMaybe<ParameterType>;
};

export type PreviewSnapshot = {
  __typename?: 'PreviewSnapshot';
  id: Scalars['ID']['output'];
  name?: Maybe<Scalars['String']['output']>;
  timestamp: Scalars['DateTime']['output'];
  updates: Array<Scalars['Int']['output']>;
  version: Scalars['Int']['output'];
};

export type Project = Node & {
  __typename?: 'Project';
  basicAuthPassword: Scalars['String']['output'];
  basicAuthUsername: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  deployment?: Maybe<Deployment>;
  description: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  isArchived: Scalars['Boolean']['output'];
  isBasicAuthActive: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  parameters: Array<Parameter>;
  sharedToken?: Maybe<Scalars['String']['output']>;
  updatedAt: Scalars['DateTime']['output'];
  version: Scalars['Int']['output'];
  workspace?: Maybe<Workspace>;
  workspaceId: Scalars['ID']['output'];
};

export type ProjectConnection = {
  __typename?: 'ProjectConnection';
  nodes: Array<Maybe<Project>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export type ProjectDocument = Node & {
  __typename?: 'ProjectDocument';
  id: Scalars['ID']['output'];
  timestamp: Scalars['DateTime']['output'];
  updates: Array<Scalars['Int']['output']>;
  version: Scalars['Int']['output'];
};

export type ProjectPayload = {
  __typename?: 'ProjectPayload';
  project: Project;
};

export type ProjectSharingInfoPayload = {
  __typename?: 'ProjectSharingInfoPayload';
  projectId: Scalars['ID']['output'];
  sharingToken?: Maybe<Scalars['String']['output']>;
};

export type ProjectSnapshot = {
  __typename?: 'ProjectSnapshot';
  timestamp: Scalars['DateTime']['output'];
  updates: Array<Scalars['Int']['output']>;
  version: Scalars['Int']['output'];
};

export type ProjectSnapshotMetadata = {
  __typename?: 'ProjectSnapshotMetadata';
  timestamp: Scalars['DateTime']['output'];
  version: Scalars['Int']['output'];
};

export enum ProjectSortField {
  CreatedAt = 'CREATED_AT',
  Name = 'NAME',
  UpdatedAt = 'UPDATED_AT'
}

export type Query = {
  __typename?: 'Query';
  assets: AssetConnection;
  cmsAsset?: Maybe<CmsAsset>;
  cmsAssets: CmsAssetsConnection;
  cmsItems: CmsItemsConnection;
  cmsModel?: Maybe<CmsModel>;
  cmsModelExportUrl: Scalars['String']['output'];
  cmsModels: CmsModelsConnection;
  cmsProject?: Maybe<CmsProject>;
  cmsProjects: Array<CmsProject>;
  deploymentByVersion?: Maybe<Deployment>;
  deploymentHead?: Maybe<Deployment>;
  deploymentVersions: Array<Deployment>;
  deployments: DeploymentConnection;
  job?: Maybe<Job>;
  jobs: JobConnection;
  latestProjectSnapshot?: Maybe<ProjectDocument>;
  me?: Maybe<Me>;
  node?: Maybe<Node>;
  nodeExecution?: Maybe<NodeExecution>;
  nodes: Array<Maybe<Node>>;
  parameters: Array<Parameter>;
  projectHistory: Array<ProjectSnapshotMetadata>;
  projectSharingInfo: ProjectSharingInfoPayload;
  projectSnapshot: ProjectSnapshot;
  projects: ProjectConnection;
  searchUser?: Maybe<User>;
  sharedProject: SharedProjectPayload;
  triggers: TriggerConnection;
};


export type QueryAssetsArgs = {
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
  sort?: InputMaybe<AssetSortType>;
  workspaceId: Scalars['ID']['input'];
};


export type QueryCmsAssetArgs = {
  assetId: Scalars['ID']['input'];
};


export type QueryCmsAssetsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  projectId: Scalars['ID']['input'];
};


export type QueryCmsItemsArgs = {
  keyword?: InputMaybe<Scalars['String']['input']>;
  modelId: Scalars['ID']['input'];
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  projectId: Scalars['ID']['input'];
};


export type QueryCmsModelArgs = {
  modelIdOrAlias: Scalars['ID']['input'];
  projectIdOrAlias: Scalars['ID']['input'];
};


export type QueryCmsModelExportUrlArgs = {
  exportType?: InputMaybe<CmsExportType>;
  modelId: Scalars['ID']['input'];
  projectId: Scalars['ID']['input'];
};


export type QueryCmsModelsArgs = {
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  projectId: Scalars['ID']['input'];
};


export type QueryCmsProjectArgs = {
  projectIdOrAlias: Scalars['ID']['input'];
};


export type QueryCmsProjectsArgs = {
  keyword?: InputMaybe<Scalars['String']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
  publicOnly?: InputMaybe<Scalars['Boolean']['input']>;
  workspaceIds: Array<Scalars['ID']['input']>;
};


export type QueryDeploymentByVersionArgs = {
  input: GetByVersionInput;
};


export type QueryDeploymentHeadArgs = {
  input: GetHeadInput;
};


export type QueryDeploymentVersionsArgs = {
  projectId?: InputMaybe<Scalars['ID']['input']>;
  workspaceId: Scalars['ID']['input'];
};


export type QueryDeploymentsArgs = {
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
  workspaceId: Scalars['ID']['input'];
};


export type QueryJobArgs = {
  id: Scalars['ID']['input'];
};


export type QueryJobsArgs = {
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
  workspaceId: Scalars['ID']['input'];
};


export type QueryLatestProjectSnapshotArgs = {
  projectId: Scalars['ID']['input'];
};


export type QueryNodeArgs = {
  id: Scalars['ID']['input'];
  type: NodeType;
};


export type QueryNodeExecutionArgs = {
  jobId: Scalars['ID']['input'];
  nodeId: Scalars['String']['input'];
};


export type QueryNodesArgs = {
  id: Array<Scalars['ID']['input']>;
  type: NodeType;
};


export type QueryParametersArgs = {
  projectId: Scalars['ID']['input'];
};


export type QueryProjectHistoryArgs = {
  projectId: Scalars['ID']['input'];
};


export type QueryProjectSharingInfoArgs = {
  projectId: Scalars['ID']['input'];
};


export type QueryProjectSnapshotArgs = {
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
};


export type QueryProjectsArgs = {
  includeArchived?: InputMaybe<Scalars['Boolean']['input']>;
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
  workspaceId: Scalars['ID']['input'];
};


export type QuerySearchUserArgs = {
  nameOrEmail: Scalars['String']['input'];
};


export type QuerySharedProjectArgs = {
  token: Scalars['String']['input'];
};


export type QueryTriggersArgs = {
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
  workspaceId: Scalars['ID']['input'];
};

export type RemoveMemberFromWorkspaceInput = {
  userId: Scalars['ID']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type RemoveMemberFromWorkspacePayload = {
  __typename?: 'RemoveMemberFromWorkspacePayload';
  workspace: Workspace;
};

export type RemoveMyAuthInput = {
  auth: Scalars['String']['input'];
};

export type RemoveParameterInput = {
  paramId: Scalars['ID']['input'];
};

export type RemoveParametersInput = {
  paramIds: Array<Scalars['ID']['input']>;
};

export enum Role {
  Maintainer = 'MAINTAINER',
  Owner = 'OWNER',
  Reader = 'READER',
  Writer = 'WRITER'
}

export type RunProjectInput = {
  file: Scalars['Upload']['input'];
  projectId: Scalars['ID']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type RunProjectPayload = {
  __typename?: 'RunProjectPayload';
  job: Job;
};

export type ShareProjectInput = {
  projectId: Scalars['ID']['input'];
};

export type ShareProjectPayload = {
  __typename?: 'ShareProjectPayload';
  projectId: Scalars['ID']['output'];
  sharingUrl: Scalars['String']['output'];
};

export type SharedProjectPayload = {
  __typename?: 'SharedProjectPayload';
  project: Project;
};

export type SignupInput = {
  lang?: InputMaybe<Scalars['Lang']['input']>;
  secret?: InputMaybe<Scalars['String']['input']>;
  userId?: InputMaybe<Scalars['ID']['input']>;
  workspaceId?: InputMaybe<Scalars['ID']['input']>;
};

export type SignupPayload = {
  __typename?: 'SignupPayload';
  user: User;
};

export type Subscription = {
  __typename?: 'Subscription';
  jobStatus: JobStatus;
  logs?: Maybe<Log>;
  nodeStatus: NodeStatus;
  userFacingLogs?: Maybe<UserFacingLog>;
};


export type SubscriptionJobStatusArgs = {
  jobId: Scalars['ID']['input'];
};


export type SubscriptionLogsArgs = {
  jobId: Scalars['ID']['input'];
};


export type SubscriptionNodeStatusArgs = {
  jobId: Scalars['ID']['input'];
  nodeId: Scalars['String']['input'];
};


export type SubscriptionUserFacingLogsArgs = {
  jobId: Scalars['ID']['input'];
};

export enum Theme {
  Dark = 'DARK',
  Default = 'DEFAULT',
  Light = 'LIGHT'
}

export type TimeDriverInput = {
  interval: TimeInterval;
};

export enum TimeInterval {
  EveryDay = 'EVERY_DAY',
  EveryHour = 'EVERY_HOUR',
  EveryMonth = 'EVERY_MONTH',
  EveryWeek = 'EVERY_WEEK'
}

export type Trigger = Node & {
  __typename?: 'Trigger';
  authToken?: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['DateTime']['output'];
  deployment: Deployment;
  deploymentId: Scalars['ID']['output'];
  description: Scalars['String']['output'];
  eventSource: EventSourceType;
  id: Scalars['ID']['output'];
  lastTriggered?: Maybe<Scalars['DateTime']['output']>;
  timeInterval?: Maybe<TimeInterval>;
  updatedAt: Scalars['DateTime']['output'];
  workspace?: Maybe<Workspace>;
  workspaceId: Scalars['ID']['output'];
};

export type TriggerConnection = {
  __typename?: 'TriggerConnection';
  nodes: Array<Maybe<Trigger>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export enum TriggerSortField {
  CreatedAt = 'CREATED_AT',
  Description = 'DESCRIPTION',
  LastTriggered = 'LAST_TRIGGERED',
  UpdatedAt = 'UPDATED_AT'
}

export type UnshareProjectInput = {
  projectId: Scalars['ID']['input'];
};

export type UnshareProjectPayload = {
  __typename?: 'UnshareProjectPayload';
  projectId: Scalars['ID']['output'];
};

export type UpdateAssetInput = {
  assetId: Scalars['ID']['input'];
  name?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateAssetPayload = {
  __typename?: 'UpdateAssetPayload';
  asset: Asset;
};

export type UpdateDeploymentInput = {
  deploymentId: Scalars['ID']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
  file?: InputMaybe<Scalars['Upload']['input']>;
};

export type UpdateMeInput = {
  email?: InputMaybe<Scalars['String']['input']>;
  lang?: InputMaybe<Scalars['Lang']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
  password?: InputMaybe<Scalars['String']['input']>;
  passwordConfirmation?: InputMaybe<Scalars['String']['input']>;
};

export type UpdateMePayload = {
  __typename?: 'UpdateMePayload';
  me: Me;
};

export type UpdateMemberOfWorkspaceInput = {
  role: Role;
  userId: Scalars['ID']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type UpdateMemberOfWorkspacePayload = {
  __typename?: 'UpdateMemberOfWorkspacePayload';
  workspace: Workspace;
};

export type UpdateParameterInput = {
  config?: InputMaybe<Scalars['JSON']['input']>;
  defaultValue: Scalars['Any']['input'];
  name: Scalars['String']['input'];
  public: Scalars['Boolean']['input'];
  required: Scalars['Boolean']['input'];
  type: ParameterType;
};

export type UpdateParameterOrderInput = {
  newIndex: Scalars['Int']['input'];
  paramId: Scalars['ID']['input'];
};

export type UpdateProjectInput = {
  archived?: InputMaybe<Scalars['Boolean']['input']>;
  basicAuthPassword?: InputMaybe<Scalars['String']['input']>;
  basicAuthUsername?: InputMaybe<Scalars['String']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  isBasicAuthActive?: InputMaybe<Scalars['Boolean']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
  projectId: Scalars['ID']['input'];
};

export type UpdateTriggerInput = {
  apiDriverInput?: InputMaybe<ApiDriverInput>;
  deploymentId?: InputMaybe<Scalars['ID']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  timeDriverInput?: InputMaybe<TimeDriverInput>;
  triggerId: Scalars['ID']['input'];
};

export type UpdateWorkspaceInput = {
  name: Scalars['String']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type UpdateWorkspacePayload = {
  __typename?: 'UpdateWorkspacePayload';
  workspace: Workspace;
};

export type User = Node & {
  __typename?: 'User';
  email: Scalars['String']['output'];
  host?: Maybe<Scalars['String']['output']>;
  id: Scalars['ID']['output'];
  metadata: UserMetadata;
  name: Scalars['String']['output'];
};

export type UserFacingLog = {
  __typename?: 'UserFacingLog';
  jobId: Scalars['ID']['output'];
  level: UserFacingLogLevel;
  message: Scalars['String']['output'];
  metadata?: Maybe<Scalars['JSON']['output']>;
  nodeId?: Maybe<Scalars['ID']['output']>;
  nodeName?: Maybe<Scalars['String']['output']>;
  timestamp: Scalars['DateTime']['output'];
};

export enum UserFacingLogLevel {
  Error = 'ERROR',
  Info = 'INFO',
  Success = 'SUCCESS'
}

export type UserMetadata = {
  __typename?: 'UserMetadata';
  description?: Maybe<Scalars['String']['output']>;
  lang: Scalars['Lang']['output'];
  photoURL?: Maybe<Scalars['String']['output']>;
  theme: Theme;
  website?: Maybe<Scalars['String']['output']>;
};

export type Workspace = Node & {
  __typename?: 'Workspace';
  assets: AssetConnection;
  id: Scalars['ID']['output'];
  members: Array<WorkspaceMember>;
  name: Scalars['String']['output'];
  personal: Scalars['Boolean']['output'];
  projects: ProjectConnection;
};


export type WorkspaceAssetsArgs = {
  pagination?: InputMaybe<Pagination>;
};


export type WorkspaceProjectsArgs = {
  includeArchived?: InputMaybe<Scalars['Boolean']['input']>;
  pagination?: InputMaybe<Pagination>;
};

export type WorkspaceMember = {
  __typename?: 'WorkspaceMember';
  role: Role;
  user?: Maybe<User>;
  userId: Scalars['ID']['output'];
};

export type GetAssetsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
}>;


export type GetAssetsQuery = { __typename?: 'Query', assets: { __typename?: 'AssetConnection', totalCount: number, nodes: Array<{ __typename?: 'Asset', id: string, workspaceId: string, createdAt: any, fileName: string, size: any, contentType: string, name: string, url: string, uuid: string, flatFiles: boolean, public: boolean, archiveExtractionStatus?: ArchiveExtractionStatus | null } | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type CreateAssetMutationVariables = Exact<{
  input: CreateAssetInput;
}>;


export type CreateAssetMutation = { __typename?: 'Mutation', createAsset?: { __typename?: 'CreateAssetPayload', asset: { __typename?: 'Asset', id: string, workspaceId: string, createdAt: any, fileName: string, size: any, contentType: string, name: string, url: string, uuid: string, flatFiles: boolean, public: boolean, archiveExtractionStatus?: ArchiveExtractionStatus | null } } | null };

export type CreateAssetUploadMutationVariables = Exact<{
  input: CreateAssetUploadInput;
}>;


export type CreateAssetUploadMutation = { __typename?: 'Mutation', createAssetUpload?: { __typename?: 'CreateAssetUploadPayload', token: string, url: string, contentType?: string | null, contentLength: number, contentEncoding?: string | null, next?: string | null } | null };

export type UpdateAssetMutationVariables = Exact<{
  input: UpdateAssetInput;
}>;


export type UpdateAssetMutation = { __typename?: 'Mutation', updateAsset?: { __typename?: 'UpdateAssetPayload', asset: { __typename?: 'Asset', id: string, workspaceId: string, createdAt: any, fileName: string, size: any, contentType: string, name: string, url: string, uuid: string, flatFiles: boolean, public: boolean, archiveExtractionStatus?: ArchiveExtractionStatus | null } } | null };

export type DeleteAssetMutationVariables = Exact<{
  input: DeleteAssetInput;
}>;


export type DeleteAssetMutation = { __typename?: 'Mutation', deleteAsset?: { __typename?: 'DeleteAssetPayload', assetId: string } | null };

export type GetCmsProjectByIdOrAliasQueryVariables = Exact<{
  projectIdOrAlias: Scalars['ID']['input'];
}>;


export type GetCmsProjectByIdOrAliasQuery = { __typename?: 'Query', cmsProject?: { __typename?: 'CMSProject', id: string, name: string, alias: string, description?: string | null, license?: string | null, readme?: string | null, workspaceId: string, visibility: CmsVisibility, topics: Array<string>, starCount: number, createdAt: any, updatedAt: any } | null };

export type GetCmsProjectsQueryVariables = Exact<{
  workspaceIds: Array<Scalars['ID']['input']> | Scalars['ID']['input'];
  keyword?: InputMaybe<Scalars['String']['input']>;
  publicOnly?: InputMaybe<Scalars['Boolean']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
}>;


export type GetCmsProjectsQuery = { __typename?: 'Query', cmsProjects: Array<{ __typename?: 'CMSProject', id: string, name: string, alias: string, description?: string | null, license?: string | null, readme?: string | null, workspaceId: string, visibility: CmsVisibility, topics: Array<string>, starCount: number, createdAt: any, updatedAt: any }> };

export type GetCmsModelsQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
}>;


export type GetCmsModelsQuery = { __typename?: 'Query', cmsModels: { __typename?: 'CMSModelsConnection', totalCount: number, models: Array<{ __typename?: 'CMSModel', id: string, projectId: string, name: string, description: string, editorUrl: string, key: string, publicApiEp: string, createdAt: any, updatedAt: any, schema: { __typename?: 'CMSSchema', schemaId: string, fields: Array<{ __typename?: 'CMSSchemaField', fieldId: string, key: string, type: CmsSchemaFieldType, name: string, description?: string | null }> } }> } };

export type GetCmsItemsQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
  modelId: Scalars['ID']['input'];
  keyword?: InputMaybe<Scalars['String']['input']>;
  page?: InputMaybe<Scalars['Int']['input']>;
  pageSize?: InputMaybe<Scalars['Int']['input']>;
}>;


export type GetCmsItemsQuery = { __typename?: 'Query', cmsItems: { __typename?: 'CMSItemsConnection', totalCount: number, items: Array<{ __typename?: 'CMSItem', id: string, fields: any, createdAt: any, updatedAt: any }> } };

export type GetCmsAssetQueryVariables = Exact<{
  assetId: Scalars['ID']['input'];
}>;


export type GetCmsAssetQuery = { __typename?: 'Query', cmsAsset?: { __typename?: 'CMSAsset', id: string, uuid: string, projectId: string, filename: string, size: number, previewType?: string | null, url: string, archiveExtractionStatus?: string | null, public: boolean, createdAt: any } | null };

export type GetCmsModelExportUrlQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
  modelId: Scalars['ID']['input'];
  exportType: CmsExportType;
}>;


export type GetCmsModelExportUrlQuery = { __typename?: 'Query', cmsModelExportUrl: string };

export type CreateDeploymentMutationVariables = Exact<{
  input: CreateDeploymentInput;
}>;


export type CreateDeploymentMutation = { __typename?: 'Mutation', createDeployment?: { __typename?: 'DeploymentPayload', deployment: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } } | null };

export type UpdateDeploymentMutationVariables = Exact<{
  input: UpdateDeploymentInput;
}>;


export type UpdateDeploymentMutation = { __typename?: 'Mutation', updateDeployment?: { __typename?: 'DeploymentPayload', deployment: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } } | null };

export type DeleteDeploymentMutationVariables = Exact<{
  input: DeleteDeploymentInput;
}>;


export type DeleteDeploymentMutation = { __typename?: 'Mutation', deleteDeployment?: { __typename?: 'DeleteDeploymentPayload', deploymentId: string } | null };

export type ExecuteDeploymentMutationVariables = Exact<{
  input: ExecuteDeploymentInput;
}>;


export type ExecuteDeploymentMutation = { __typename?: 'Mutation', executeDeployment?: { __typename?: 'JobPayload', job: { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, outputURLs?: Array<string> | null, userFacingLogsURL?: string | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } } | null };

export type GetDeploymentsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
}>;


export type GetDeploymentsQuery = { __typename?: 'Query', deployments: { __typename?: 'DeploymentConnection', totalCount: number, nodes: Array<{ __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetDeploymentHeadQueryVariables = Exact<{
  input: GetHeadInput;
}>;


export type GetDeploymentHeadQuery = { __typename?: 'Query', deploymentHead?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null };

export type GetLatestProjectSnapshotQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetLatestProjectSnapshotQuery = { __typename?: 'Query', latestProjectSnapshot?: { __typename?: 'ProjectDocument', id: string, timestamp: any, updates: Array<number>, version: number } | null };

export type GetProjectSnapshotQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
}>;


export type GetProjectSnapshotQuery = { __typename?: 'Query', projectSnapshot: { __typename?: 'ProjectSnapshot', timestamp: any, updates: Array<number>, version: number } };

export type GetProjectHistoryQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetProjectHistoryQuery = { __typename?: 'Query', projectHistory: Array<{ __typename?: 'ProjectSnapshotMetadata', timestamp: any, version: number }> };

export type PreviewSnapshotMutationVariables = Exact<{
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
}>;


export type PreviewSnapshotMutation = { __typename?: 'Mutation', previewSnapshot?: { __typename?: 'PreviewSnapshot', id: string, timestamp: any, updates: Array<number>, version: number } | null };

export type RollbackProjectMutationVariables = Exact<{
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
}>;


export type RollbackProjectMutation = { __typename?: 'Mutation', rollbackProject?: { __typename?: 'ProjectDocument', id: string, timestamp: any, updates: Array<number>, version: number } | null };

export type SaveSnapshotMutationVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type SaveSnapshotMutation = { __typename?: 'Mutation', saveSnapshot: boolean };

export type ProjectFragment = { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null };

export type WorkspaceFragment = { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> };

export type ParameterFragment = { __typename?: 'Parameter', id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config?: any | null, createdAt: any, updatedAt: any };

export type DeploymentFragment = { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null };

export type TriggerFragment = { __typename?: 'Trigger', id: string, createdAt: any, updatedAt: any, lastTriggered?: any | null, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken?: string | null, timeInterval?: TimeInterval | null, description: string, deployment: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } };

export type NodeExecutionFragment = { __typename?: 'NodeExecution', id: string, nodeId: string, jobId: string, status: NodeStatus, createdAt?: any | null, startedAt?: any | null, completedAt?: any | null };

export type JobFragment = { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, outputURLs?: Array<string> | null, userFacingLogsURL?: string | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null };

export type AssetFragment = { __typename?: 'Asset', id: string, workspaceId: string, createdAt: any, fileName: string, size: any, contentType: string, name: string, url: string, uuid: string, flatFiles: boolean, public: boolean, archiveExtractionStatus?: ArchiveExtractionStatus | null };

export type ProjectDocumentFragment = { __typename?: 'ProjectDocument', id: string, timestamp: any, updates: Array<number>, version: number };

export type ProjectSnapshotMetadataFragment = { __typename?: 'ProjectSnapshotMetadata', timestamp: any, version: number };

export type ProjectSnapshotFragment = { __typename?: 'ProjectSnapshot', timestamp: any, updates: Array<number>, version: number };

export type UserFacingLogFragment = { __typename?: 'UserFacingLog', jobId: string, timestamp: any, nodeId?: string | null, nodeName?: string | null, level: UserFacingLogLevel, message: string };

export type CmsProjectFragment = { __typename?: 'CMSProject', id: string, name: string, alias: string, description?: string | null, license?: string | null, readme?: string | null, workspaceId: string, visibility: CmsVisibility, topics: Array<string>, starCount: number, createdAt: any, updatedAt: any };

export type CmsModelFragment = { __typename?: 'CMSModel', id: string, projectId: string, name: string, description: string, editorUrl: string, key: string, publicApiEp: string, createdAt: any, updatedAt: any, schema: { __typename?: 'CMSSchema', schemaId: string, fields: Array<{ __typename?: 'CMSSchemaField', fieldId: string, key: string, type: CmsSchemaFieldType, name: string, description?: string | null }> } };

export type CmsItemFragment = { __typename?: 'CMSItem', id: string, fields: any, createdAt: any, updatedAt: any };

export type CmsAssetFragment = { __typename?: 'CMSAsset', id: string, uuid: string, projectId: string, filename: string, size: number, previewType?: string | null, url: string, archiveExtractionStatus?: string | null, public: boolean, createdAt: any };

export type GetJobsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
}>;


export type GetJobsQuery = { __typename?: 'Query', jobs: { __typename?: 'JobConnection', totalCount: number, nodes: Array<{ __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, outputURLs?: Array<string> | null, userFacingLogsURL?: string | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetJobQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetJobQuery = { __typename?: 'Query', job?: { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, outputURLs?: Array<string> | null, userFacingLogsURL?: string | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } | null };

export type GetNodeExecutionQueryVariables = Exact<{
  jobId: Scalars['ID']['input'];
  nodeId: Scalars['String']['input'];
}>;


export type GetNodeExecutionQuery = { __typename?: 'Query', nodeExecution?: { __typename?: 'NodeExecution', id: string, nodeId: string, jobId: string, status: NodeStatus, createdAt?: any | null, startedAt?: any | null, completedAt?: any | null } | null };

export type CancelJobMutationVariables = Exact<{
  input: CancelJobInput;
}>;


export type CancelJobMutation = { __typename?: 'Mutation', cancelJob: { __typename?: 'CancelJobPayload', job?: { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, outputURLs?: Array<string> | null, userFacingLogsURL?: string | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } | null } };

export type CreateProjectMutationVariables = Exact<{
  input: CreateProjectInput;
}>;


export type CreateProjectMutation = { __typename?: 'Mutation', createProject?: { __typename?: 'ProjectPayload', project: { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null } } | null };

export type GetProjectsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
}>;


export type GetProjectsQuery = { __typename?: 'Query', projects: { __typename?: 'ProjectConnection', totalCount: number, nodes: Array<{ __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null } | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetProjectByIdQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetProjectByIdQuery = { __typename?: 'Query', node?:
    | { __typename: 'Asset' }
    | { __typename: 'Deployment' }
    | { __typename: 'Job' }
    | { __typename: 'NodeExecution' }
    | { __typename: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null }
    | { __typename: 'ProjectDocument' }
    | { __typename: 'Trigger' }
    | { __typename: 'User' }
    | { __typename: 'Workspace' }
   | null };

export type UpdateProjectMutationVariables = Exact<{
  input: UpdateProjectInput;
}>;


export type UpdateProjectMutation = { __typename?: 'Mutation', updateProject?: { __typename?: 'ProjectPayload', project: { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null } } | null };

export type DeleteProjectMutationVariables = Exact<{
  input: DeleteProjectInput;
}>;


export type DeleteProjectMutation = { __typename?: 'Mutation', deleteProject?: { __typename?: 'DeleteProjectPayload', projectId: string } | null };

export type RunProjectMutationVariables = Exact<{
  input: RunProjectInput;
}>;


export type RunProjectMutation = { __typename?: 'Mutation', runProject?: { __typename?: 'RunProjectPayload', job: { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, outputURLs?: Array<string> | null, userFacingLogsURL?: string | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } } | null };

export type CopyProjectMutationVariables = Exact<{
  projectId: Scalars['ID']['input'];
  source: Scalars['ID']['input'];
}>;


export type CopyProjectMutation = { __typename?: 'Mutation', copyProject: boolean };

export type ImportProjectMutationVariables = Exact<{
  projectId: Scalars['ID']['input'];
  data: Scalars['Bytes']['input'];
}>;


export type ImportProjectMutation = { __typename?: 'Mutation', importProject: boolean };

export type GetProjectParametersQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetProjectParametersQuery = { __typename?: 'Query', parameters: Array<{ __typename?: 'Parameter', id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config?: any | null, createdAt: any, updatedAt: any }> };

export type CreateProjectVariableMutationVariables = Exact<{
  projectId: Scalars['ID']['input'];
  input: DeclareParameterInput;
}>;


export type CreateProjectVariableMutation = { __typename?: 'Mutation', declareParameter: { __typename?: 'Parameter', id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config?: any | null, createdAt: any, updatedAt: any } };

export type UpdateProjectVariableMutationVariables = Exact<{
  paramId: Scalars['ID']['input'];
  input: UpdateParameterInput;
}>;


export type UpdateProjectVariableMutation = { __typename?: 'Mutation', updateParameter: { __typename?: 'Parameter', id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config?: any | null, createdAt: any, updatedAt: any } };

export type UpdateProjectVariablesMutationVariables = Exact<{
  input: ParameterBatchInput;
}>;


export type UpdateProjectVariablesMutation = { __typename?: 'Mutation', updateParameters: Array<{ __typename?: 'Parameter', id: string, projectId: string, index: number, name: string, defaultValue: any, type: ParameterType, required: boolean, public: boolean, config?: any | null, createdAt: any, updatedAt: any }> };

export type DeleteProjectVariableMutationVariables = Exact<{
  input: RemoveParameterInput;
}>;


export type DeleteProjectVariableMutation = { __typename?: 'Mutation', removeParameter: boolean };

export type DeleteProjectVariablesMutationVariables = Exact<{
  input: RemoveParametersInput;
}>;


export type DeleteProjectVariablesMutation = { __typename?: 'Mutation', removeParameters: boolean };

export type GetSharedProjectQueryVariables = Exact<{
  token: Scalars['String']['input'];
}>;


export type GetSharedProjectQuery = { __typename?: 'Query', sharedProject: { __typename?: 'SharedProjectPayload', project: { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null } } };

export type GetSharedProjectInfoQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetSharedProjectInfoQuery = { __typename?: 'Query', projectSharingInfo: { __typename?: 'ProjectSharingInfoPayload', projectId: string, sharingToken?: string | null } };

export type ShareProjectMutationVariables = Exact<{
  input: ShareProjectInput;
}>;


export type ShareProjectMutation = { __typename?: 'Mutation', shareProject?: { __typename?: 'ShareProjectPayload', projectId: string, sharingUrl: string } | null };

export type UnshareProjectMutationVariables = Exact<{
  input: UnshareProjectInput;
}>;


export type UnshareProjectMutation = { __typename?: 'Mutation', unshareProject?: { __typename?: 'UnshareProjectPayload', projectId: string } | null };

export type OnJobStatusChangeSubscriptionVariables = Exact<{
  jobId: Scalars['ID']['input'];
}>;


export type OnJobStatusChangeSubscription = { __typename?: 'Subscription', jobStatus: JobStatus };

export type OnNodeStatusChangeSubscriptionVariables = Exact<{
  jobId: Scalars['ID']['input'];
  nodeId: Scalars['String']['input'];
}>;


export type OnNodeStatusChangeSubscription = { __typename?: 'Subscription', nodeStatus: NodeStatus };

export type UserFacingLogsSubscriptionVariables = Exact<{
  jobId: Scalars['ID']['input'];
}>;


export type UserFacingLogsSubscription = { __typename?: 'Subscription', userFacingLogs?: { __typename?: 'UserFacingLog', jobId: string, timestamp: any, nodeId?: string | null, nodeName?: string | null, level: UserFacingLogLevel, message: string } | null };

export type CreateTriggerMutationVariables = Exact<{
  input: CreateTriggerInput;
}>;


export type CreateTriggerMutation = { __typename?: 'Mutation', createTrigger: { __typename?: 'Trigger', id: string, createdAt: any, updatedAt: any, lastTriggered?: any | null, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken?: string | null, timeInterval?: TimeInterval | null, description: string, deployment: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } } };

export type UpdateTriggerMutationVariables = Exact<{
  input: UpdateTriggerInput;
}>;


export type UpdateTriggerMutation = { __typename?: 'Mutation', updateTrigger: { __typename?: 'Trigger', id: string, createdAt: any, updatedAt: any, lastTriggered?: any | null, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken?: string | null, timeInterval?: TimeInterval | null, description: string, deployment: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } } };

export type DeleteTriggerMutationVariables = Exact<{
  triggerId: Scalars['ID']['input'];
}>;


export type DeleteTriggerMutation = { __typename?: 'Mutation', deleteTrigger: boolean };

export type GetTriggersQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination: PageBasedPagination;
}>;


export type GetTriggersQuery = { __typename?: 'Query', triggers: { __typename?: 'TriggerConnection', totalCount: number, nodes: Array<{ __typename?: 'Trigger', id: string, createdAt: any, updatedAt: any, lastTriggered?: any | null, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken?: string | null, timeInterval?: TimeInterval | null, description: string, deployment: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } } | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetMeQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMeQuery = { __typename?: 'Query', me?: { __typename?: 'Me', id: string, name: string, email: string, myWorkspaceId: string, lang: any } | null };

export type GetMeAndWorkspacesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMeAndWorkspacesQuery = { __typename?: 'Query', me?: { __typename?: 'Me', id: string, name: string, email: string, myWorkspaceId: string, lang: any, workspaces: Array<{ __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> }> } | null };

export type SearchUserQueryVariables = Exact<{
  email: Scalars['String']['input'];
}>;


export type SearchUserQuery = { __typename?: 'Query', searchUser?: { __typename?: 'User', id: string, name: string, email: string } | null };

export type UpdateMeMutationVariables = Exact<{
  input: UpdateMeInput;
}>;


export type UpdateMeMutation = { __typename?: 'Mutation', updateMe?: { __typename?: 'UpdateMePayload', me: { __typename?: 'Me', id: string, name: string, email: string, lang: any } } | null };

export type CreateWorkspaceMutationVariables = Exact<{
  input: CreateWorkspaceInput;
}>;


export type CreateWorkspaceMutation = { __typename?: 'Mutation', createWorkspace?: { __typename?: 'CreateWorkspacePayload', workspace: { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } } | null };

export type GetWorkspacesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetWorkspacesQuery = { __typename?: 'Query', me?: { __typename?: 'Me', id: string, workspaces: Array<{ __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> }> } | null };

export type GetWorkspaceByIdQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
}>;


export type GetWorkspaceByIdQuery = { __typename?: 'Query', node?:
    | { __typename: 'Asset' }
    | { __typename: 'Deployment' }
    | { __typename: 'Job' }
    | { __typename: 'NodeExecution' }
    | { __typename: 'Project' }
    | { __typename: 'ProjectDocument' }
    | { __typename: 'Trigger' }
    | { __typename: 'User' }
    | { __typename: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> }
   | null };

export type UpdateWorkspaceMutationVariables = Exact<{
  input: UpdateWorkspaceInput;
}>;


export type UpdateWorkspaceMutation = { __typename?: 'Mutation', updateWorkspace?: { __typename?: 'UpdateWorkspacePayload', workspace: { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } } | null };

export type DeleteWorkspaceMutationVariables = Exact<{
  input: DeleteWorkspaceInput;
}>;


export type DeleteWorkspaceMutation = { __typename?: 'Mutation', deleteWorkspace?: { __typename?: 'DeleteWorkspacePayload', workspaceId: string } | null };

export type AddMemberToWorkspaceMutationVariables = Exact<{
  input: AddMemberToWorkspaceInput;
}>;


export type AddMemberToWorkspaceMutation = { __typename?: 'Mutation', addMemberToWorkspace?: { __typename?: 'AddMemberToWorkspacePayload', workspace: { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } } | null };

export type RemoveMemberFromWorkspaceMutationVariables = Exact<{
  input: RemoveMemberFromWorkspaceInput;
}>;


export type RemoveMemberFromWorkspaceMutation = { __typename?: 'Mutation', removeMemberFromWorkspace?: { __typename?: 'RemoveMemberFromWorkspacePayload', workspace: { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } } | null };

export type UpdateMemberOfWorkspaceMutationVariables = Exact<{
  input: UpdateMemberOfWorkspaceInput;
}>;


export type UpdateMemberOfWorkspaceMutation = { __typename?: 'Mutation', updateMemberOfWorkspace?: { __typename?: 'UpdateMemberOfWorkspacePayload', workspace: { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } } | null };

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
}
    ${DeploymentFragmentDoc}`;
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
export const GetProjectParametersDocument = gql`
    query GetProjectParameters($projectId: ID!) {
  parameters(projectId: $projectId) {
    ...Parameter
  }
}
    ${ParameterFragmentDoc}`;
export const CreateProjectVariableDocument = gql`
    mutation CreateProjectVariable($projectId: ID!, $input: DeclareParameterInput!) {
  declareParameter(projectId: $projectId, input: $input) {
    ...Parameter
  }
}
    ${ParameterFragmentDoc}`;
export const UpdateProjectVariableDocument = gql`
    mutation UpdateProjectVariable($paramId: ID!, $input: UpdateParameterInput!) {
  updateParameter(paramId: $paramId, input: $input) {
    ...Parameter
  }
}
    ${ParameterFragmentDoc}`;
export const UpdateProjectVariablesDocument = gql`
    mutation UpdateProjectVariables($input: ParameterBatchInput!) {
  updateParameters(input: $input) {
    ...Parameter
  }
}
    ${ParameterFragmentDoc}`;
export const DeleteProjectVariableDocument = gql`
    mutation DeleteProjectVariable($input: RemoveParameterInput!) {
  removeParameter(input: $input)
}
    `;
export const DeleteProjectVariablesDocument = gql`
    mutation DeleteProjectVariables($input: RemoveParametersInput!) {
  removeParameters(input: $input)
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
    GetProjectParameters(variables: GetProjectParametersQueryVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<GetProjectParametersQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetProjectParametersQuery>({ document: GetProjectParametersDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'GetProjectParameters', 'query', variables);
    },
    CreateProjectVariable(variables: CreateProjectVariableMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<CreateProjectVariableMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateProjectVariableMutation>({ document: CreateProjectVariableDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'CreateProjectVariable', 'mutation', variables);
    },
    UpdateProjectVariable(variables: UpdateProjectVariableMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateProjectVariableMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateProjectVariableMutation>({ document: UpdateProjectVariableDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateProjectVariable', 'mutation', variables);
    },
    UpdateProjectVariables(variables: UpdateProjectVariablesMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<UpdateProjectVariablesMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateProjectVariablesMutation>({ document: UpdateProjectVariablesDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'UpdateProjectVariables', 'mutation', variables);
    },
    DeleteProjectVariable(variables: DeleteProjectVariableMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteProjectVariableMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteProjectVariableMutation>({ document: DeleteProjectVariableDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteProjectVariable', 'mutation', variables);
    },
    DeleteProjectVariables(variables: DeleteProjectVariablesMutationVariables, requestHeaders?: GraphQLClientRequestHeaders, signal?: RequestInit['signal']): Promise<DeleteProjectVariablesMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteProjectVariablesMutation>({ document: DeleteProjectVariablesDocument, variables, requestHeaders: { ...requestHeaders, ...wrappedRequestHeaders }, signal }), 'DeleteProjectVariables', 'mutation', variables);
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