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
  flushProjectToGcs?: Maybe<Scalars['Boolean']['output']>;
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


export type MutationFlushProjectToGcsArgs = {
  projectId: Scalars['ID']['input'];
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

export type ProjectSnapshotMetadata = {
  __typename?: 'ProjectSnapshotMetadata';
  timestamp: Scalars['DateTime']['output'];
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
  nodeExecution?: Maybe<NodeExecution>;
  nodes: Array<Maybe<Node>>;
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


export type QueryNodeExecutionArgs = {
  jobId: Scalars['ID']['input'];
  nodeId: Scalars['String']['input'];
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


export type QueryProjectSnapshotArgs = {
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
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
  nodeStatus: NodeStatus;
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


export type ExecuteDeploymentMutation = { __typename?: 'Mutation', executeDeployment?: { __typename?: 'JobPayload', job: { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, logsURL?: string | null, outputURLs?: Array<string> | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } } | null };

export type GetDeploymentsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
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

export type GetProjectHistoryQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetProjectHistoryQuery = { __typename?: 'Query', projectHistory: Array<{ __typename?: 'ProjectSnapshotMetadata', timestamp: any, version: number }> };

export type RollbackProjectMutationVariables = Exact<{
  projectId: Scalars['ID']['input'];
  version: Scalars['Int']['input'];
}>;


export type RollbackProjectMutation = { __typename?: 'Mutation', rollbackProject?: { __typename?: 'ProjectDocument', id: string, timestamp: any, updates: Array<number>, version: number } | null };

export type ProjectFragment = { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null };

export type WorkspaceFragment = { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> };

export type DeploymentFragment = { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null };

export type TriggerFragment = { __typename?: 'Trigger', id: string, createdAt: any, updatedAt: any, lastTriggered?: any | null, workspaceId: string, deploymentId: string, eventSource: EventSourceType, authToken?: string | null, timeInterval?: TimeInterval | null, description: string, deployment: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } };

export type NodeExecutionFragment = { __typename?: 'NodeExecution', id: string, nodeId: string, jobId: string, status: NodeStatus, createdAt?: any | null, startedAt?: any | null, completedAt?: any | null };

export type JobFragment = { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, logsURL?: string | null, outputURLs?: Array<string> | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null };

export type ProjectDocumentFragment = { __typename?: 'ProjectDocument', id: string, timestamp: any, updates: Array<number>, version: number };

export type ProjectSnapshotMetadataFragment = { __typename?: 'ProjectSnapshotMetadata', timestamp: any, version: number };

export type ProjectSnapshotFragment = { __typename?: 'ProjectSnapshot', timestamp: any, updates: Array<number>, version: number };

export type LogFragment = { __typename?: 'Log', jobId: string, nodeId?: string | null, timestamp: any, logLevel: LogLevel, message: string };

export type GetJobsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  pagination: PageBasedPagination;
}>;


export type GetJobsQuery = { __typename?: 'Query', jobs: { __typename?: 'JobConnection', totalCount: number, nodes: Array<{ __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, logsURL?: string | null, outputURLs?: Array<string> | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetJobQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetJobQuery = { __typename?: 'Query', job?: { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, logsURL?: string | null, outputURLs?: Array<string> | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } | null };

export type GetNodeExecutionQueryVariables = Exact<{
  jobId: Scalars['ID']['input'];
  nodeId: Scalars['String']['input'];
}>;


export type GetNodeExecutionQuery = { __typename?: 'Query', nodeExecution?: { __typename?: 'NodeExecution', id: string, nodeId: string, jobId: string, status: NodeStatus, createdAt?: any | null, startedAt?: any | null, completedAt?: any | null } | null };

export type CancelJobMutationVariables = Exact<{
  input: CancelJobInput;
}>;


export type CancelJobMutation = { __typename?: 'Mutation', cancelJob: { __typename?: 'CancelJobPayload', job?: { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, logsURL?: string | null, outputURLs?: Array<string> | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } | null } };

export type CreateProjectMutationVariables = Exact<{
  input: CreateProjectInput;
}>;


export type CreateProjectMutation = { __typename?: 'Mutation', createProject?: { __typename?: 'ProjectPayload', project: { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null } } | null };

export type GetProjectsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  pagination: PageBasedPagination;
}>;


export type GetProjectsQuery = { __typename?: 'Query', projects: { __typename?: 'ProjectConnection', totalCount: number, nodes: Array<{ __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null } | null>, pageInfo: { __typename?: 'PageInfo', totalCount: number, currentPage?: number | null, totalPages?: number | null } } };

export type GetProjectByIdQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetProjectByIdQuery = { __typename?: 'Query', node?: { __typename: 'Asset' } | { __typename: 'Deployment' } | { __typename: 'Job' } | { __typename: 'NodeExecution' } | { __typename: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string, sharedToken?: string | null, deployment?: { __typename?: 'Deployment', id: string, projectId?: string | null, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null } | { __typename: 'ProjectDocument' } | { __typename: 'Trigger' } | { __typename: 'User' } | { __typename: 'Workspace' } | null };

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


export type RunProjectMutation = { __typename?: 'Mutation', runProject?: { __typename?: 'RunProjectPayload', job: { __typename?: 'Job', id: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null, logsURL?: string | null, outputURLs?: Array<string> | null, debug?: boolean | null, deployment?: { __typename?: 'Deployment', id: string, description: string } | null } } | null };

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

export type RealTimeLogsSubscriptionVariables = Exact<{
  jobId: Scalars['ID']['input'];
}>;


export type RealTimeLogsSubscription = { __typename?: 'Subscription', logs?: { __typename?: 'Log', jobId: string, nodeId?: string | null, timestamp: any, logLevel: LogLevel, message: string } | null };

export type OnNodeStatusChangeSubscriptionVariables = Exact<{
  jobId: Scalars['ID']['input'];
  nodeId: Scalars['String']['input'];
}>;


export type OnNodeStatusChangeSubscription = { __typename?: 'Subscription', nodeStatus: NodeStatus };

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


export type GetWorkspaceByIdQuery = { __typename?: 'Query', node?: { __typename: 'Asset' } | { __typename: 'Deployment' } | { __typename: 'Job' } | { __typename: 'NodeExecution' } | { __typename: 'Project' } | { __typename: 'ProjectDocument' } | { __typename: 'Trigger' } | { __typename: 'User' } | { __typename: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } | null };

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
  logsURL
  outputURLs
  debug
  deployment {
    id
    description
  }
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
export const LogFragmentDoc = gql`
    fragment Log on Log {
  jobId
  nodeId
  timestamp
  logLevel
  message
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
    query GetDeployments($workspaceId: ID!, $pagination: PageBasedPagination!) {
  deployments(workspaceId: $workspaceId, pagination: $pagination) {
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
export const GetProjectHistoryDocument = gql`
    query GetProjectHistory($projectId: ID!) {
  projectHistory(projectId: $projectId) {
    timestamp
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
export const GetJobsDocument = gql`
    query GetJobs($workspaceId: ID!, $pagination: PageBasedPagination!) {
  jobs(workspaceId: $workspaceId, pagination: $pagination) {
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
    query GetProjects($workspaceId: ID!, $pagination: PageBasedPagination!) {
  projects(workspaceId: $workspaceId, pagination: $pagination) {
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
export const RealTimeLogsDocument = gql`
    subscription RealTimeLogs($jobId: ID!) {
  logs(jobId: $jobId) {
    jobId
    nodeId
    timestamp
    logLevel
    message
  }
}
    `;
export const OnNodeStatusChangeDocument = gql`
    subscription OnNodeStatusChange($jobId: ID!, $nodeId: String!) {
  nodeStatus(jobId: $jobId, nodeId: $nodeId)
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
    query GetTriggers($workspaceId: ID!, $pagination: PageBasedPagination!) {
  triggers(workspaceId: $workspaceId, pagination: $pagination) {
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
    CreateDeployment(variables: CreateDeploymentMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<CreateDeploymentMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateDeploymentMutation>(CreateDeploymentDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'CreateDeployment', 'mutation', variables);
    },
    UpdateDeployment(variables: UpdateDeploymentMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<UpdateDeploymentMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateDeploymentMutation>(UpdateDeploymentDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'UpdateDeployment', 'mutation', variables);
    },
    DeleteDeployment(variables: DeleteDeploymentMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<DeleteDeploymentMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteDeploymentMutation>(DeleteDeploymentDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'DeleteDeployment', 'mutation', variables);
    },
    ExecuteDeployment(variables: ExecuteDeploymentMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<ExecuteDeploymentMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<ExecuteDeploymentMutation>(ExecuteDeploymentDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'ExecuteDeployment', 'mutation', variables);
    },
    GetDeployments(variables: GetDeploymentsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetDeploymentsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetDeploymentsQuery>(GetDeploymentsDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetDeployments', 'query', variables);
    },
    GetDeploymentHead(variables: GetDeploymentHeadQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetDeploymentHeadQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetDeploymentHeadQuery>(GetDeploymentHeadDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetDeploymentHead', 'query', variables);
    },
    GetLatestProjectSnapshot(variables: GetLatestProjectSnapshotQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetLatestProjectSnapshotQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetLatestProjectSnapshotQuery>(GetLatestProjectSnapshotDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetLatestProjectSnapshot', 'query', variables);
    },
    GetProjectHistory(variables: GetProjectHistoryQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetProjectHistoryQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetProjectHistoryQuery>(GetProjectHistoryDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetProjectHistory', 'query', variables);
    },
    RollbackProject(variables: RollbackProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<RollbackProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<RollbackProjectMutation>(RollbackProjectDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'RollbackProject', 'mutation', variables);
    },
    GetJobs(variables: GetJobsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetJobsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetJobsQuery>(GetJobsDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetJobs', 'query', variables);
    },
    GetJob(variables: GetJobQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetJobQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetJobQuery>(GetJobDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetJob', 'query', variables);
    },
    GetNodeExecution(variables: GetNodeExecutionQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetNodeExecutionQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetNodeExecutionQuery>(GetNodeExecutionDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetNodeExecution', 'query', variables);
    },
    CancelJob(variables: CancelJobMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<CancelJobMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CancelJobMutation>(CancelJobDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'CancelJob', 'mutation', variables);
    },
    CreateProject(variables: CreateProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<CreateProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateProjectMutation>(CreateProjectDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'CreateProject', 'mutation', variables);
    },
    GetProjects(variables: GetProjectsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetProjectsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetProjectsQuery>(GetProjectsDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetProjects', 'query', variables);
    },
    GetProjectById(variables: GetProjectByIdQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetProjectByIdQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetProjectByIdQuery>(GetProjectByIdDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetProjectById', 'query', variables);
    },
    UpdateProject(variables: UpdateProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<UpdateProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateProjectMutation>(UpdateProjectDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'UpdateProject', 'mutation', variables);
    },
    DeleteProject(variables: DeleteProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<DeleteProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteProjectMutation>(DeleteProjectDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'DeleteProject', 'mutation', variables);
    },
    RunProject(variables: RunProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<RunProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<RunProjectMutation>(RunProjectDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'RunProject', 'mutation', variables);
    },
    GetSharedProject(variables: GetSharedProjectQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetSharedProjectQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetSharedProjectQuery>(GetSharedProjectDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetSharedProject', 'query', variables);
    },
    GetSharedProjectInfo(variables: GetSharedProjectInfoQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetSharedProjectInfoQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetSharedProjectInfoQuery>(GetSharedProjectInfoDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetSharedProjectInfo', 'query', variables);
    },
    ShareProject(variables: ShareProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<ShareProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<ShareProjectMutation>(ShareProjectDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'ShareProject', 'mutation', variables);
    },
    UnshareProject(variables: UnshareProjectMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<UnshareProjectMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UnshareProjectMutation>(UnshareProjectDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'UnshareProject', 'mutation', variables);
    },
    OnJobStatusChange(variables: OnJobStatusChangeSubscriptionVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<OnJobStatusChangeSubscription> {
      return withWrapper((wrappedRequestHeaders) => client.request<OnJobStatusChangeSubscription>(OnJobStatusChangeDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'OnJobStatusChange', 'subscription', variables);
    },
    RealTimeLogs(variables: RealTimeLogsSubscriptionVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<RealTimeLogsSubscription> {
      return withWrapper((wrappedRequestHeaders) => client.request<RealTimeLogsSubscription>(RealTimeLogsDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'RealTimeLogs', 'subscription', variables);
    },
    OnNodeStatusChange(variables: OnNodeStatusChangeSubscriptionVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<OnNodeStatusChangeSubscription> {
      return withWrapper((wrappedRequestHeaders) => client.request<OnNodeStatusChangeSubscription>(OnNodeStatusChangeDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'OnNodeStatusChange', 'subscription', variables);
    },
    CreateTrigger(variables: CreateTriggerMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<CreateTriggerMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateTriggerMutation>(CreateTriggerDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'CreateTrigger', 'mutation', variables);
    },
    UpdateTrigger(variables: UpdateTriggerMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<UpdateTriggerMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateTriggerMutation>(UpdateTriggerDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'UpdateTrigger', 'mutation', variables);
    },
    DeleteTrigger(variables: DeleteTriggerMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<DeleteTriggerMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteTriggerMutation>(DeleteTriggerDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'DeleteTrigger', 'mutation', variables);
    },
    GetTriggers(variables: GetTriggersQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetTriggersQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetTriggersQuery>(GetTriggersDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetTriggers', 'query', variables);
    },
    GetMe(variables?: GetMeQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetMeQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetMeQuery>(GetMeDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetMe', 'query', variables);
    },
    GetMeAndWorkspaces(variables?: GetMeAndWorkspacesQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetMeAndWorkspacesQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetMeAndWorkspacesQuery>(GetMeAndWorkspacesDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetMeAndWorkspaces', 'query', variables);
    },
    SearchUser(variables: SearchUserQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<SearchUserQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<SearchUserQuery>(SearchUserDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'SearchUser', 'query', variables);
    },
    UpdateMe(variables: UpdateMeMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<UpdateMeMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateMeMutation>(UpdateMeDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'UpdateMe', 'mutation', variables);
    },
    CreateWorkspace(variables: CreateWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<CreateWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateWorkspaceMutation>(CreateWorkspaceDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'CreateWorkspace', 'mutation', variables);
    },
    GetWorkspaces(variables?: GetWorkspacesQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetWorkspacesQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetWorkspacesQuery>(GetWorkspacesDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetWorkspaces', 'query', variables);
    },
    GetWorkspaceById(variables: GetWorkspaceByIdQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetWorkspaceByIdQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetWorkspaceByIdQuery>(GetWorkspaceByIdDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetWorkspaceById', 'query', variables);
    },
    UpdateWorkspace(variables: UpdateWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<UpdateWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateWorkspaceMutation>(UpdateWorkspaceDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'UpdateWorkspace', 'mutation', variables);
    },
    DeleteWorkspace(variables: DeleteWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<DeleteWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeleteWorkspaceMutation>(DeleteWorkspaceDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'DeleteWorkspace', 'mutation', variables);
    },
    AddMemberToWorkspace(variables: AddMemberToWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<AddMemberToWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<AddMemberToWorkspaceMutation>(AddMemberToWorkspaceDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'AddMemberToWorkspace', 'mutation', variables);
    },
    RemoveMemberFromWorkspace(variables: RemoveMemberFromWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<RemoveMemberFromWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<RemoveMemberFromWorkspaceMutation>(RemoveMemberFromWorkspaceDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'RemoveMemberFromWorkspace', 'mutation', variables);
    },
    UpdateMemberOfWorkspace(variables: UpdateMemberOfWorkspaceMutationVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<UpdateMemberOfWorkspaceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<UpdateMemberOfWorkspaceMutation>(UpdateMemberOfWorkspaceDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'UpdateMemberOfWorkspace', 'mutation', variables);
    }
  };
}
export type Sdk = ReturnType<typeof getSdk>;