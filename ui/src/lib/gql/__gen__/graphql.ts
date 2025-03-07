/* eslint-disable */
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  Any: { input: any; output: any; }
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

export type Asset = Node & {
  __typename?: 'Asset';
  Workspace?: Maybe<Workspace>;
  contentType: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['ID']['output'];
  name: Scalars['String']['output'];
  size: Scalars['FileSize']['output'];
  url: Scalars['String']['output'];
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

export type CancelJobInput = {
  jobId: Scalars['ID']['input'];
};

export type CancelJobPayload = {
  __typename?: 'CancelJobPayload';
  job?: Maybe<Job>;
};

export type CreateAssetInput = {
  file: Scalars['Upload']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type CreateAssetPayload = {
  __typename?: 'CreateAssetPayload';
  asset: Asset;
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
  index?: InputMaybe<Scalars['Int']['input']>;
  name: Scalars['String']['input'];
  required: Scalars['Boolean']['input'];
  type: ParameterType;
  value?: InputMaybe<Scalars['Any']['input']>;
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
  createAsset?: Maybe<CreateAssetPayload>;
  createDeployment?: Maybe<DeploymentPayload>;
  createProject?: Maybe<ProjectPayload>;
  createTrigger: Trigger;
  createWorkspace?: Maybe<CreateWorkspacePayload>;
  declareParameter: Parameter;
  deleteDeployment?: Maybe<DeleteDeploymentPayload>;
  deleteMe?: Maybe<DeleteMePayload>;
  deleteProject?: Maybe<DeleteProjectPayload>;
  deleteTrigger: Scalars['Boolean']['output'];
  deleteWorkspace?: Maybe<DeleteWorkspacePayload>;
  executeDeployment?: Maybe<JobPayload>;
  removeAsset?: Maybe<RemoveAssetPayload>;
  removeMemberFromWorkspace?: Maybe<RemoveMemberFromWorkspacePayload>;
  removeMyAuth?: Maybe<UpdateMePayload>;
  removeParameter: Scalars['Boolean']['output'];
  rollbackProject?: Maybe<ProjectDocument>;
  runProject?: Maybe<RunProjectPayload>;
  shareProject?: Maybe<ShareProjectPayload>;
  signup?: Maybe<SignupPayload>;
  unshareProject?: Maybe<UnshareProjectPayload>;
  updateDeployment?: Maybe<DeploymentPayload>;
  updateMe?: Maybe<UpdateMePayload>;
  updateMemberOfWorkspace?: Maybe<UpdateMemberOfWorkspacePayload>;
  updateParameterOrder: Array<Parameter>;
  updateParameterValue: Parameter;
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


export type MutationCreateAssetArgs = {
  input: CreateAssetInput;
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


export type MutationRemoveAssetArgs = {
  input: RemoveAssetInput;
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


export type MutationRollbackProjectArgs = {
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
};


export type MutationRunProjectArgs = {
  input: RunProjectInput;
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


export type MutationUpdateDeploymentArgs = {
  input: UpdateDeploymentInput;
};


export type MutationUpdateMeArgs = {
  input: UpdateMeInput;
};


export type MutationUpdateMemberOfWorkspaceArgs = {
  input: UpdateMemberOfWorkspaceInput;
};


export type MutationUpdateParameterOrderArgs = {
  input: UpdateParameterOrderInput;
  projectId: Scalars['ID']['input'];
};


export type MutationUpdateParameterValueArgs = {
  input: UpdateParameterValueInput;
  paramId: Scalars['ID']['input'];
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
  createdAt: Scalars['DateTime']['output'];
  id: Scalars['ID']['output'];
  index: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  projectId: Scalars['ID']['output'];
  required: Scalars['Boolean']['output'];
  type: ParameterType;
  updatedAt: Scalars['DateTime']['output'];
  value: Scalars['Any']['output'];
};

export enum ParameterType {
  AttributeName = 'ATTRIBUTE_NAME',
  Choice = 'CHOICE',
  Color = 'COLOR',
  CoordinateSystem = 'COORDINATE_SYSTEM',
  DatabaseConnection = 'DATABASE_CONNECTION',
  Datetime = 'DATETIME',
  FileFolder = 'FILE_FOLDER',
  Geometry = 'GEOMETRY',
  Message = 'MESSAGE',
  Number = 'NUMBER',
  Password = 'PASSWORD',
  ReprojectionFile = 'REPROJECTION_FILE',
  Text = 'TEXT',
  WebConnection = 'WEB_CONNECTION',
  YesNo = 'YES_NO'
}

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

export type Query = {
  __typename?: 'Query';
  assets: AssetConnection;
  deploymentByVersion?: Maybe<Deployment>;
  deploymentHead?: Maybe<Deployment>;
  deploymentVersions: Array<Deployment>;
  deployments: DeploymentConnection;
  job?: Maybe<Job>;
  jobs: JobConnection;
  latestProjectSnapshot?: Maybe<ProjectDocument>;
  me?: Maybe<Me>;
  node?: Maybe<Node>;
  nodes: Array<Maybe<Node>>;
  projectHistory: Array<ProjectSnapshot>;
  projectSharingInfo: ProjectSharingInfoPayload;
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
  pagination: PageBasedPagination;
  workspaceId: Scalars['ID']['input'];
};


export type QueryJobArgs = {
  id: Scalars['ID']['input'];
};


export type QueryJobsArgs = {
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


export type QueryNodesArgs = {
  id: Array<Scalars['ID']['input']>;
  type: NodeType;
};


export type QueryProjectHistoryArgs = {
  projectId: Scalars['ID']['input'];
};


export type QueryProjectSharingInfoArgs = {
  projectId: Scalars['ID']['input'];
};


export type QueryProjectsArgs = {
  includeArchived?: InputMaybe<Scalars['Boolean']['input']>;
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
  pagination: PageBasedPagination;
  workspaceId: Scalars['ID']['input'];
};

export type RemoveAssetInput = {
  assetId: Scalars['ID']['input'];
};

export type RemoveAssetPayload = {
  __typename?: 'RemoveAssetPayload';
  assetId: Scalars['ID']['output'];
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
  workspace: Workspace;
};

export type Subscription = {
  __typename?: 'Subscription';
  jobStatus: JobStatus;
  logs?: Maybe<Log>;
};


export type SubscriptionJobStatusArgs = {
  jobId: Scalars['ID']['input'];
};


export type SubscriptionLogsArgs = {
  jobId: Scalars['ID']['input'];
};

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

export type UnshareProjectInput = {
  projectId: Scalars['ID']['input'];
};

export type UnshareProjectPayload = {
  __typename?: 'UnshareProjectPayload';
  projectId: Scalars['ID']['output'];
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

export type UpdateParameterOrderInput = {
  newIndex: Scalars['Int']['input'];
  paramId: Scalars['ID']['input'];
};

export type UpdateParameterValueInput = {
  value: Scalars['Any']['input'];
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
  name: Scalars['String']['output'];
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

export type CreateDeploymentMutationVariables = Exact<{
  input: CreateDeploymentInput;
}>;


export type CreateDeploymentMutation = { __typename?: 'Mutation', createDeployment?: { __typename?: 'DeploymentPayload', deployment: (
      { __typename?: 'Deployment' }
      & { ' $fragmentRefs'?: { 'DeploymentFragment': DeploymentFragment } }
    ) } | null };

export type UpdateDeploymentMutationVariables = Exact<{
  input: UpdateDeploymentInput;
}>;


export type UpdateDeploymentMutation = { __typename?: 'Mutation', updateDeployment?: { __typename?: 'DeploymentPayload', deployment: (
      { __typename?: 'Deployment' }
      & { ' $fragmentRefs'?: { 'DeploymentFragment': DeploymentFragment } }
    ) } | null };

export type DeleteDeploymentMutationVariables = Exact<{
  input: DeleteDeploymentInput;
}>;


export type DeleteDeploymentMutation = { __typename?: 'Mutation', deleteDeployment?: { __typename?: 'DeleteDeploymentPayload', deploymentId: string } | null };

export type ExecuteDeploymentMutationVariables = Exact<{
  input: ExecuteDeploymentInput;
}>;


export type ExecuteDeploymentMutation = { __typename?: 'Mutation', executeDeployment?: { __typename?: 'JobPayload', job: (
      { __typename?: 'Job' }
      & { ' $fragmentRefs'?: { 'JobFragment': JobFragment } }
    ) } | null };

export type GetDeploymentsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  pagination: PageBasedPagination;
}>;


export type GetDeploymentsQuery = { __typename?: 'Query', deployments: { __typename?: 'DeploymentConnection', totalCount: number, nodes: Array<(
      { __typename?: 'Deployment' }
      & { ' $fragmentRefs'?: { 'DeploymentFragment': DeploymentFragment } }
    ) | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type ProjectFragment = { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: (
    { __typename?: 'Deployment' }
    & { ' $fragmentRefs'?: { 'DeploymentFragment': DeploymentFragment } }
  ) | null } & { ' $fragmentName'?: 'ProjectFragment' };

export type DeploymentFragment = { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } & { ' $fragmentName'?: 'DeploymentFragment' };

export type TriggerFragment = { __typename?: 'Trigger', id: string, createdAt: any, updatedAt: any, lastTriggered?: any | null, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken?: string | null, timeInterval?: TimeInterval | null, description: string, deployment: (
    { __typename?: 'Deployment' }
    & { ' $fragmentRefs'?: { 'DeploymentFragment': DeploymentFragment } }
  ) } & { ' $fragmentName'?: 'TriggerFragment' };

export type JobFragment = { __typename?: 'Job', id: string, deploymentId: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, logsURL?: string | null, outputURLs?: Array<string> | null, deployment?: (
    { __typename?: 'Deployment' }
    & { ' $fragmentRefs'?: { 'DeploymentFragment': DeploymentFragment } }
  ) | null } & { ' $fragmentName'?: 'JobFragment' };

export type LogFragment = { __typename?: 'Log', jobId: string, nodeId?: string | null, timestamp: any, logLevel: LogLevel, message: string } & { ' $fragmentName'?: 'LogFragment' };

export type GetJobsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  pagination: PageBasedPagination;
}>;


export type GetJobsQuery = { __typename?: 'Query', jobs: { __typename?: 'JobConnection', totalCount: number, nodes: Array<(
      { __typename?: 'Job' }
      & { ' $fragmentRefs'?: { 'JobFragment': JobFragment } }
    ) | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetJobQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetJobQuery = { __typename?: 'Query', job?: (
    { __typename?: 'Job' }
    & { ' $fragmentRefs'?: { 'JobFragment': JobFragment } }
  ) | null };

export type CancelJobMutationVariables = Exact<{
  input: CancelJobInput;
}>;


export type CancelJobMutation = { __typename?: 'Mutation', cancelJob: { __typename?: 'CancelJobPayload', job?: (
      { __typename?: 'Job' }
      & { ' $fragmentRefs'?: { 'JobFragment': JobFragment } }
    ) | null } };

export type CreateProjectMutationVariables = Exact<{
  input: CreateProjectInput;
}>;


export type CreateProjectMutation = { __typename?: 'Mutation', createProject?: { __typename?: 'ProjectPayload', project: (
      { __typename?: 'Project' }
      & { ' $fragmentRefs'?: { 'ProjectFragment': ProjectFragment } }
    ) } | null };

export type GetProjectsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  pagination: PageBasedPagination;
}>;


export type GetProjectsQuery = { __typename?: 'Query', projects: { __typename?: 'ProjectConnection', totalCount: number, nodes: Array<(
      { __typename?: 'Project' }
      & { ' $fragmentRefs'?: { 'ProjectFragment': ProjectFragment } }
    ) | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetProjectByIdQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetProjectByIdQuery = { __typename?: 'Query', node?: { __typename: 'Asset' } | { __typename: 'Deployment' } | { __typename: 'Job' } | (
    { __typename: 'Project' }
    & { ' $fragmentRefs'?: { 'ProjectFragment': ProjectFragment } }
  ) | { __typename: 'ProjectDocument' } | { __typename: 'Trigger' } | { __typename: 'User' } | { __typename: 'Workspace' } | null };

export type UpdateProjectMutationVariables = Exact<{
  input: UpdateProjectInput;
}>;


export type UpdateProjectMutation = { __typename?: 'Mutation', updateProject?: { __typename?: 'ProjectPayload', project: (
      { __typename?: 'Project' }
      & { ' $fragmentRefs'?: { 'ProjectFragment': ProjectFragment } }
    ) } | null };

export type DeleteProjectMutationVariables = Exact<{
  input: DeleteProjectInput;
}>;


export type DeleteProjectMutation = { __typename?: 'Mutation', deleteProject?: { __typename?: 'DeleteProjectPayload', projectId: string } | null };

export type RunProjectMutationVariables = Exact<{
  input: RunProjectInput;
}>;


export type RunProjectMutation = { __typename?: 'Mutation', runProject?: { __typename?: 'RunProjectPayload', job: { __typename?: 'Job', id: string, status: JobStatus, startedAt: any } } | null };

export type GetSharedProjectQueryVariables = Exact<{
  token: Scalars['String']['input'];
}>;


export type GetSharedProjectQuery = { __typename?: 'Query', sharedProject: { __typename?: 'SharedProjectPayload', project: (
      { __typename?: 'Project' }
      & { ' $fragmentRefs'?: { 'ProjectFragment': ProjectFragment } }
    ) } };

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

export type RealTimeLogsSubscriptionVariables = Exact<{
  jobId: Scalars['ID']['input'];
}>;


export type RealTimeLogsSubscription = { __typename?: 'Subscription', logs?: { __typename?: 'Log', jobId: string, nodeId?: string | null, timestamp: any, logLevel: LogLevel, message: string } | null };

export type CreateTriggerMutationVariables = Exact<{
  input: CreateTriggerInput;
}>;


export type CreateTriggerMutation = { __typename?: 'Mutation', createTrigger: (
    { __typename?: 'Trigger' }
    & { ' $fragmentRefs'?: { 'TriggerFragment': TriggerFragment } }
  ) };

export type UpdateTriggerMutationVariables = Exact<{
  input: UpdateTriggerInput;
}>;


export type UpdateTriggerMutation = { __typename?: 'Mutation', updateTrigger: (
    { __typename?: 'Trigger' }
    & { ' $fragmentRefs'?: { 'TriggerFragment': TriggerFragment } }
  ) };

export type DeleteTriggerMutationVariables = Exact<{
  triggerId: Scalars['ID']['input'];
}>;


export type DeleteTriggerMutation = { __typename?: 'Mutation', deleteTrigger: boolean };

export type GetTriggersQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  pagination: PageBasedPagination;
}>;


export type GetTriggersQuery = { __typename?: 'Query', triggers: { __typename?: 'TriggerConnection', totalCount: number, nodes: Array<(
      { __typename?: 'Trigger' }
      & { ' $fragmentRefs'?: { 'TriggerFragment': TriggerFragment } }
    ) | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetMeQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMeQuery = { __typename?: 'Query', me?: { __typename?: 'Me', id: string, name: string, email: string, myWorkspaceId: string, lang: any } | null };

export type SearchUserQueryVariables = Exact<{
  email: Scalars['String']['input'];
}>;


export type SearchUserQuery = { __typename?: 'Query', searchUser?: { __typename?: 'User', id: string, name: string, email: string } | null };

export type UpdateMeMutationVariables = Exact<{
  input: UpdateMeInput;
}>;


export type UpdateMeMutation = { __typename?: 'Mutation', updateMe?: { __typename?: 'UpdateMePayload', me: { __typename?: 'Me', id: string, name: string, email: string, lang: any } } | null };

export type WorkspaceFragment = { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } & { ' $fragmentName'?: 'WorkspaceFragment' };

export type CreateWorkspaceMutationVariables = Exact<{
  input: CreateWorkspaceInput;
}>;


export type CreateWorkspaceMutation = { __typename?: 'Mutation', createWorkspace?: { __typename?: 'CreateWorkspacePayload', workspace: (
      { __typename?: 'Workspace' }
      & { ' $fragmentRefs'?: { 'WorkspaceFragment': WorkspaceFragment } }
    ) } | null };

export type GetWorkspacesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetWorkspacesQuery = { __typename?: 'Query', me?: { __typename?: 'Me', id: string, workspaces: Array<(
      { __typename?: 'Workspace' }
      & { ' $fragmentRefs'?: { 'WorkspaceFragment': WorkspaceFragment } }
    )> } | null };

export type GetWorkspaceByIdQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
}>;


export type GetWorkspaceByIdQuery = { __typename?: 'Query', node?: { __typename: 'Asset' } | { __typename: 'Deployment' } | { __typename: 'Job' } | { __typename: 'Project' } | { __typename: 'ProjectDocument' } | { __typename: 'Trigger' } | { __typename: 'User' } | (
    { __typename: 'Workspace' }
    & { ' $fragmentRefs'?: { 'WorkspaceFragment': WorkspaceFragment } }
  ) | null };

export type UpdateWorkspaceMutationVariables = Exact<{
  input: UpdateWorkspaceInput;
}>;


export type UpdateWorkspaceMutation = { __typename?: 'Mutation', updateWorkspace?: { __typename?: 'UpdateWorkspacePayload', workspace: (
      { __typename?: 'Workspace' }
      & { ' $fragmentRefs'?: { 'WorkspaceFragment': WorkspaceFragment } }
    ) } | null };

export type DeleteWorkspaceMutationVariables = Exact<{
  input: DeleteWorkspaceInput;
}>;


export type DeleteWorkspaceMutation = { __typename?: 'Mutation', deleteWorkspace?: { __typename?: 'DeleteWorkspacePayload', workspaceId: string } | null };

export type AddMemberToWorkspaceMutationVariables = Exact<{
  input: AddMemberToWorkspaceInput;
}>;


export type AddMemberToWorkspaceMutation = { __typename?: 'Mutation', addMemberToWorkspace?: { __typename?: 'AddMemberToWorkspacePayload', workspace: (
      { __typename?: 'Workspace' }
      & { ' $fragmentRefs'?: { 'WorkspaceFragment': WorkspaceFragment } }
    ) } | null };

export type RemoveMemberFromWorkspaceMutationVariables = Exact<{
  input: RemoveMemberFromWorkspaceInput;
}>;


export type RemoveMemberFromWorkspaceMutation = { __typename?: 'Mutation', removeMemberFromWorkspace?: { __typename?: 'RemoveMemberFromWorkspacePayload', workspace: (
      { __typename?: 'Workspace' }
      & { ' $fragmentRefs'?: { 'WorkspaceFragment': WorkspaceFragment } }
    ) } | null };

export type UpdateMemberOfWorkspaceMutationVariables = Exact<{
  input: UpdateMemberOfWorkspaceInput;
}>;


export type UpdateMemberOfWorkspaceMutation = { __typename?: 'Mutation', updateMemberOfWorkspace?: { __typename?: 'UpdateMemberOfWorkspacePayload', workspace: (
      { __typename?: 'Workspace' }
      & { ' $fragmentRefs'?: { 'WorkspaceFragment': WorkspaceFragment } }
    ) } | null };

export const DeploymentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<DeploymentFragment, unknown>;
export const ProjectFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Project"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"sharedToken"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<ProjectFragment, unknown>;
export const TriggerFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Trigger"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Trigger"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastTriggered"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"eventSource"}},{"kind":"Field","name":{"kind":"Name","value":"authToken"}},{"kind":"Field","name":{"kind":"Name","value":"timeInterval"}},{"kind":"Field","name":{"kind":"Name","value":"description"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<TriggerFragment, unknown>;
export const JobFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"startedAt"}},{"kind":"Field","name":{"kind":"Name","value":"completedAt"}},{"kind":"Field","name":{"kind":"Name","value":"logsURL"}},{"kind":"Field","name":{"kind":"Name","value":"outputURLs"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<JobFragment, unknown>;
export const LogFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Log"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Log"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobId"}},{"kind":"Field","name":{"kind":"Name","value":"nodeId"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"logLevel"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}}]} as unknown as DocumentNode<LogFragment, unknown>;
export const WorkspaceFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Workspace"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Workspace"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"personal"}},{"kind":"Field","name":{"kind":"Name","value":"members"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"role"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<WorkspaceFragment, unknown>;
export const CreateDeploymentDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateDeployment"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateDeploymentInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createDeployment"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<CreateDeploymentMutation, CreateDeploymentMutationVariables>;
export const UpdateDeploymentDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateDeployment"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateDeploymentInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateDeployment"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<UpdateDeploymentMutation, UpdateDeploymentMutationVariables>;
export const DeleteDeploymentDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteDeployment"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DeleteDeploymentInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteDeployment"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}}]}}]}}]} as unknown as DocumentNode<DeleteDeploymentMutation, DeleteDeploymentMutationVariables>;
export const ExecuteDeploymentDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ExecuteDeployment"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ExecuteDeploymentInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"executeDeployment"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"job"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"startedAt"}},{"kind":"Field","name":{"kind":"Name","value":"completedAt"}},{"kind":"Field","name":{"kind":"Name","value":"logsURL"}},{"kind":"Field","name":{"kind":"Name","value":"outputURLs"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<ExecuteDeploymentMutation, ExecuteDeploymentMutationVariables>;
export const GetDeploymentsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetDeployments"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"PageBasedPagination"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deployments"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"workspaceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"pagination"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"totalCount"}},{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"totalCount"}},{"kind":"Field","name":{"kind":"Name","value":"currentPage"}},{"kind":"Field","name":{"kind":"Name","value":"totalPages"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<GetDeploymentsQuery, GetDeploymentsQueryVariables>;
export const GetJobsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetJobs"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"PageBasedPagination"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobs"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"workspaceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"pagination"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"totalCount"}},{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"totalCount"}},{"kind":"Field","name":{"kind":"Name","value":"currentPage"}},{"kind":"Field","name":{"kind":"Name","value":"totalPages"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"startedAt"}},{"kind":"Field","name":{"kind":"Name","value":"completedAt"}},{"kind":"Field","name":{"kind":"Name","value":"logsURL"}},{"kind":"Field","name":{"kind":"Name","value":"outputURLs"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<GetJobsQuery, GetJobsQueryVariables>;
export const GetJobDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetJob"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"job"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"startedAt"}},{"kind":"Field","name":{"kind":"Name","value":"completedAt"}},{"kind":"Field","name":{"kind":"Name","value":"logsURL"}},{"kind":"Field","name":{"kind":"Name","value":"outputURLs"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<GetJobQuery, GetJobQueryVariables>;
export const CancelJobDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CancelJob"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CancelJobInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"cancelJob"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"job"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Job"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Job"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Job"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"startedAt"}},{"kind":"Field","name":{"kind":"Name","value":"completedAt"}},{"kind":"Field","name":{"kind":"Name","value":"logsURL"}},{"kind":"Field","name":{"kind":"Name","value":"outputURLs"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<CancelJobMutation, CancelJobMutationVariables>;
export const CreateProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateProjectInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Project"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Project"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"sharedToken"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<CreateProjectMutation, CreateProjectMutationVariables>;
export const GetProjectsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetProjects"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"PageBasedPagination"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projects"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"workspaceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"pagination"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"totalCount"}},{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Project"}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"totalCount"}},{"kind":"Field","name":{"kind":"Name","value":"currentPage"}},{"kind":"Field","name":{"kind":"Name","value":"totalPages"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Project"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"sharedToken"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<GetProjectsQuery, GetProjectsQueryVariables>;
export const GetProjectByIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetProjectById"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}},{"kind":"Argument","name":{"kind":"Name","value":"type"},"value":{"kind":"EnumValue","value":"PROJECT"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"Project"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Project"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"sharedToken"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<GetProjectByIdQuery, GetProjectByIdQueryVariables>;
export const UpdateProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateProjectInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Project"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Project"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"sharedToken"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<UpdateProjectMutation, UpdateProjectMutationVariables>;
export const DeleteProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DeleteProjectInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projectId"}}]}}]}}]} as unknown as DocumentNode<DeleteProjectMutation, DeleteProjectMutationVariables>;
export const RunProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"RunProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RunProjectInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"runProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"job"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"startedAt"}}]}}]}}]}}]} as unknown as DocumentNode<RunProjectMutation, RunProjectMutationVariables>;
export const GetSharedProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSharedProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"token"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sharedProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"token"},"value":{"kind":"Variable","name":{"kind":"Name","value":"token"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Project"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Project"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Project"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"sharedToken"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}}]}}]} as unknown as DocumentNode<GetSharedProjectQuery, GetSharedProjectQueryVariables>;
export const GetSharedProjectInfoDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSharedProjectInfo"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projectSharingInfo"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"projectId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"projectId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"sharingToken"}}]}}]}}]} as unknown as DocumentNode<GetSharedProjectInfoQuery, GetSharedProjectInfoQueryVariables>;
export const ShareProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ShareProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ShareProjectInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"shareProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"sharingUrl"}}]}}]}}]} as unknown as DocumentNode<ShareProjectMutation, ShareProjectMutationVariables>;
export const UnshareProjectDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UnshareProject"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UnshareProjectInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"unshareProject"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"projectId"}}]}}]}}]} as unknown as DocumentNode<UnshareProjectMutation, UnshareProjectMutationVariables>;
export const OnJobStatusChangeDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"OnJobStatusChange"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"jobId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobStatus"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"jobId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"jobId"}}}]}]}}]} as unknown as DocumentNode<OnJobStatusChangeSubscription, OnJobStatusChangeSubscriptionVariables>;
export const RealTimeLogsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"RealTimeLogs"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"jobId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"logs"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"jobId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"jobId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"jobId"}},{"kind":"Field","name":{"kind":"Name","value":"nodeId"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"logLevel"}},{"kind":"Field","name":{"kind":"Name","value":"message"}}]}}]}}]} as unknown as DocumentNode<RealTimeLogsSubscription, RealTimeLogsSubscriptionVariables>;
export const CreateTriggerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateTrigger"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateTriggerInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createTrigger"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Trigger"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Trigger"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Trigger"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastTriggered"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"eventSource"}},{"kind":"Field","name":{"kind":"Name","value":"authToken"}},{"kind":"Field","name":{"kind":"Name","value":"timeInterval"}},{"kind":"Field","name":{"kind":"Name","value":"description"}}]}}]} as unknown as DocumentNode<CreateTriggerMutation, CreateTriggerMutationVariables>;
export const UpdateTriggerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateTrigger"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateTriggerInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateTrigger"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Trigger"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Trigger"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Trigger"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastTriggered"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"eventSource"}},{"kind":"Field","name":{"kind":"Name","value":"authToken"}},{"kind":"Field","name":{"kind":"Name","value":"timeInterval"}},{"kind":"Field","name":{"kind":"Name","value":"description"}}]}}]} as unknown as DocumentNode<UpdateTriggerMutation, UpdateTriggerMutationVariables>;
export const DeleteTriggerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteTrigger"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"triggerId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteTrigger"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"triggerId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"triggerId"}}}]}]}}]} as unknown as DocumentNode<DeleteTriggerMutation, DeleteTriggerMutationVariables>;
export const GetTriggersDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetTriggers"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"PageBasedPagination"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"triggers"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"workspaceId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"pagination"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"totalCount"}},{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Trigger"}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"totalCount"}},{"kind":"Field","name":{"kind":"Name","value":"currentPage"}},{"kind":"Field","name":{"kind":"Name","value":"totalPages"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Deployment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Deployment"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"projectId"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"workflowUrl"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"version"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"project"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Trigger"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Trigger"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastTriggered"}},{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"deploymentId"}},{"kind":"Field","name":{"kind":"Name","value":"deployment"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Deployment"}}]}},{"kind":"Field","name":{"kind":"Name","value":"eventSource"}},{"kind":"Field","name":{"kind":"Name","value":"authToken"}},{"kind":"Field","name":{"kind":"Name","value":"timeInterval"}},{"kind":"Field","name":{"kind":"Name","value":"description"}}]}}]} as unknown as DocumentNode<GetTriggersQuery, GetTriggersQueryVariables>;
export const GetMeDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetMe"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"myWorkspaceId"}},{"kind":"Field","name":{"kind":"Name","value":"lang"}}]}}]}}]} as unknown as DocumentNode<GetMeQuery, GetMeQueryVariables>;
export const SearchUserDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SearchUser"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"email"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"searchUser"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"nameOrEmail"},"value":{"kind":"Variable","name":{"kind":"Name","value":"email"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"email"}}]}}]}}]} as unknown as DocumentNode<SearchUserQuery, SearchUserQueryVariables>;
export const UpdateMeDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateMe"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateMeInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateMe"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"lang"}}]}}]}}]}}]} as unknown as DocumentNode<UpdateMeMutation, UpdateMeMutationVariables>;
export const CreateWorkspaceDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateWorkspace"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CreateWorkspaceInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createWorkspace"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"workspace"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Workspace"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Workspace"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Workspace"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"personal"}},{"kind":"Field","name":{"kind":"Name","value":"members"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"role"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<CreateWorkspaceMutation, CreateWorkspaceMutationVariables>;
export const GetWorkspacesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetWorkspaces"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"me"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"workspaces"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Workspace"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Workspace"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Workspace"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"personal"}},{"kind":"Field","name":{"kind":"Name","value":"members"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"role"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<GetWorkspacesQuery, GetWorkspacesQueryVariables>;
export const GetWorkspaceByIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetWorkspaceById"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ID"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"workspaceId"}}},{"kind":"Argument","name":{"kind":"Name","value":"type"},"value":{"kind":"EnumValue","value":"WORKSPACE"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"__typename"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"Workspace"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Workspace"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Workspace"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"personal"}},{"kind":"Field","name":{"kind":"Name","value":"members"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"role"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<GetWorkspaceByIdQuery, GetWorkspaceByIdQueryVariables>;
export const UpdateWorkspaceDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateWorkspace"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateWorkspaceInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateWorkspace"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"workspace"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Workspace"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Workspace"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Workspace"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"personal"}},{"kind":"Field","name":{"kind":"Name","value":"members"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"role"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<UpdateWorkspaceMutation, UpdateWorkspaceMutationVariables>;
export const DeleteWorkspaceDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteWorkspace"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"DeleteWorkspaceInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteWorkspace"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"workspaceId"}}]}}]}}]} as unknown as DocumentNode<DeleteWorkspaceMutation, DeleteWorkspaceMutationVariables>;
export const AddMemberToWorkspaceDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"AddMemberToWorkspace"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"AddMemberToWorkspaceInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"addMemberToWorkspace"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"workspace"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Workspace"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Workspace"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Workspace"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"personal"}},{"kind":"Field","name":{"kind":"Name","value":"members"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"role"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<AddMemberToWorkspaceMutation, AddMemberToWorkspaceMutationVariables>;
export const RemoveMemberFromWorkspaceDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"RemoveMemberFromWorkspace"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RemoveMemberFromWorkspaceInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"removeMemberFromWorkspace"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"workspace"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Workspace"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Workspace"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Workspace"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"personal"}},{"kind":"Field","name":{"kind":"Name","value":"members"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"role"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<RemoveMemberFromWorkspaceMutation, RemoveMemberFromWorkspaceMutationVariables>;
export const UpdateMemberOfWorkspaceDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateMemberOfWorkspace"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"UpdateMemberOfWorkspaceInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateMemberOfWorkspace"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"workspace"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"Workspace"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"Workspace"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Workspace"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"personal"}},{"kind":"Field","name":{"kind":"Name","value":"members"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"role"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"email"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]}}]} as unknown as DocumentNode<UpdateMemberOfWorkspaceMutation, UpdateMemberOfWorkspaceMutationVariables>;