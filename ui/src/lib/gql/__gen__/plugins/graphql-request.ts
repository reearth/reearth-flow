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
  Cursor: { input: any; output: any; }
  DateTime: { input: any; output: any; }
  FileSize: { input: any; output: any; }
  Lang: { input: any; output: any; }
  URL: { input: any; output: any; }
  Upload: { input: any; output: any; }
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
  edges: Array<AssetEdge>;
  nodes: Array<Maybe<Asset>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export type AssetEdge = {
  __typename?: 'AssetEdge';
  cursor: Scalars['Cursor']['output'];
  node?: Maybe<Asset>;
};

export enum AssetSortType {
  Date = 'DATE',
  Name = 'NAME',
  Size = 'SIZE'
}

export type CreateAssetInput = {
  file: Scalars['Upload']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type CreateAssetPayload = {
  __typename?: 'CreateAssetPayload';
  asset: Asset;
};

export type CreateDeploymentInput = {
  description?: InputMaybe<Scalars['String']['input']>;
  file: Scalars['Upload']['input'];
  projectId: Scalars['ID']['input'];
  workspaceId: Scalars['ID']['input'];
};

export type CreateProjectInput = {
  archived?: InputMaybe<Scalars['Boolean']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
  workspaceId: Scalars['ID']['input'];
};

export type CreateWorkspaceInput = {
  name: Scalars['String']['input'];
};

export type CreateWorkspacePayload = {
  __typename?: 'CreateWorkspacePayload';
  workspace: Workspace;
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
  id: Scalars['ID']['output'];
  project?: Maybe<Project>;
  projectId: Scalars['ID']['output'];
  updatedAt: Scalars['DateTime']['output'];
  version: Scalars['String']['output'];
  workflowUrl: Scalars['String']['output'];
  workspace?: Maybe<Workspace>;
  workspaceId: Scalars['ID']['output'];
};

export type DeploymentConnection = {
  __typename?: 'DeploymentConnection';
  edges: Array<DeploymentEdge>;
  nodes: Array<Maybe<Deployment>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export type DeploymentEdge = {
  __typename?: 'DeploymentEdge';
  cursor: Scalars['Cursor']['output'];
  node?: Maybe<Deployment>;
};

export type DeploymentPayload = {
  __typename?: 'DeploymentPayload';
  deployment: Deployment;
};

export type ExecuteDeploymentInput = {
  deploymentId: Scalars['ID']['input'];
};

export type InputGraph = {
  edges: Array<InputWorkflowEdge>;
  id: Scalars['ID']['input'];
  name: Scalars['String']['input'];
  nodes: Array<InputWorkflowNode>;
};

export type InputWorkflow = {
  entryGraphId: Scalars['ID']['input'];
  graphs: Array<InputMaybe<InputGraph>>;
  id: Scalars['ID']['input'];
  name: Scalars['String']['input'];
  with?: InputMaybe<Scalars['Any']['input']>;
};

export type InputWorkflowEdge = {
  from: Scalars['ID']['input'];
  fromPort: Scalars['String']['input'];
  id: Scalars['ID']['input'];
  to: Scalars['ID']['input'];
  toPort: Scalars['String']['input'];
};

export type InputWorkflowNode = {
  action?: InputMaybe<Scalars['String']['input']>;
  id: Scalars['ID']['input'];
  name: Scalars['String']['input'];
  subGraphId?: InputMaybe<Scalars['ID']['input']>;
  type?: InputMaybe<Scalars['String']['input']>;
  with?: InputMaybe<Scalars['Any']['input']>;
};

export type Job = Node & {
  __typename?: 'Job';
  completedAt?: Maybe<Scalars['DateTime']['output']>;
  deployment?: Maybe<Deployment>;
  deploymentId: Scalars['ID']['output'];
  id: Scalars['ID']['output'];
  startedAt: Scalars['DateTime']['output'];
  status: JobStatus;
  workspace?: Maybe<Workspace>;
  workspaceId: Scalars['ID']['output'];
};

export type JobConnection = {
  __typename?: 'JobConnection';
  edges: Array<JobEdge>;
  nodes: Array<Maybe<Job>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export type JobEdge = {
  __typename?: 'JobEdge';
  cursor: Scalars['Cursor']['output'];
  node?: Maybe<Job>;
};

export type JobPayload = {
  __typename?: 'JobPayload';
  job: Job;
};

export enum JobStatus {
  Completed = 'COMPLETED',
  Failed = 'FAILED',
  Pending = 'PENDING',
  Running = 'RUNNING'
}

export type Me = {
  __typename?: 'Me';
  auths: Array<Scalars['String']['output']>;
  email: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  myWorkspace?: Maybe<Workspace>;
  myWorkspaceId: Scalars['ID']['output'];
  name: Scalars['String']['output'];
  workspaces: Array<Workspace>;
};

export type Mutation = {
  __typename?: 'Mutation';
  addMemberToWorkspace?: Maybe<AddMemberToWorkspacePayload>;
  createAsset?: Maybe<CreateAssetPayload>;
  createDeployment?: Maybe<DeploymentPayload>;
  createProject?: Maybe<ProjectPayload>;
  createWorkspace?: Maybe<CreateWorkspacePayload>;
  deleteDeployment?: Maybe<DeleteDeploymentPayload>;
  deleteMe?: Maybe<DeleteMePayload>;
  deleteProject?: Maybe<DeleteProjectPayload>;
  deleteWorkspace?: Maybe<DeleteWorkspacePayload>;
  executeDeployment?: Maybe<JobPayload>;
  removeAsset?: Maybe<RemoveAssetPayload>;
  removeMemberFromWorkspace?: Maybe<RemoveMemberFromWorkspacePayload>;
  removeMyAuth?: Maybe<UpdateMePayload>;
  runProject?: Maybe<RunProjectPayload>;
  signup?: Maybe<SignupPayload>;
  updateDeployment?: Maybe<DeploymentPayload>;
  updateMe?: Maybe<UpdateMePayload>;
  updateMemberOfWorkspace?: Maybe<UpdateMemberOfWorkspacePayload>;
  updateProject?: Maybe<ProjectPayload>;
  updateWorkspace?: Maybe<UpdateWorkspacePayload>;
};


export type MutationAddMemberToWorkspaceArgs = {
  input: AddMemberToWorkspaceInput;
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


export type MutationCreateWorkspaceArgs = {
  input: CreateWorkspaceInput;
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


export type MutationRunProjectArgs = {
  input: RunProjectInput;
};


export type MutationSignupArgs = {
  input: SignupInput;
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


export type MutationUpdateProjectArgs = {
  input: UpdateProjectInput;
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

export type PageInfo = {
  __typename?: 'PageInfo';
  endCursor?: Maybe<Scalars['Cursor']['output']>;
  hasNextPage: Scalars['Boolean']['output'];
  hasPreviousPage: Scalars['Boolean']['output'];
  startCursor?: Maybe<Scalars['Cursor']['output']>;
};

export type Pagination = {
  after?: InputMaybe<Scalars['Cursor']['input']>;
  before?: InputMaybe<Scalars['Cursor']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
  last?: InputMaybe<Scalars['Int']['input']>;
};

export type Project = Node & {
  __typename?: 'Project';
  basicAuthPassword: Scalars['String']['output'];
  basicAuthUsername: Scalars['String']['output'];
  createdAt: Scalars['DateTime']['output'];
  description: Scalars['String']['output'];
  id: Scalars['ID']['output'];
  isArchived: Scalars['Boolean']['output'];
  isBasicAuthActive: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  updatedAt: Scalars['DateTime']['output'];
  version: Scalars['Int']['output'];
  workspace?: Maybe<Workspace>;
  workspaceId: Scalars['ID']['output'];
};

export type ProjectConnection = {
  __typename?: 'ProjectConnection';
  edges: Array<ProjectEdge>;
  nodes: Array<Maybe<Project>>;
  pageInfo: PageInfo;
  totalCount: Scalars['Int']['output'];
};

export type ProjectEdge = {
  __typename?: 'ProjectEdge';
  cursor: Scalars['Cursor']['output'];
  node?: Maybe<Project>;
};

export type ProjectPayload = {
  __typename?: 'ProjectPayload';
  project: Project;
};

export type Query = {
  __typename?: 'Query';
  assets: AssetConnection;
  deployments: DeploymentConnection;
  job?: Maybe<Job>;
  jobs: JobConnection;
  me?: Maybe<Me>;
  node?: Maybe<Node>;
  nodes: Array<Maybe<Node>>;
  projects: ProjectConnection;
  searchUser?: Maybe<User>;
};


export type QueryAssetsArgs = {
  keyword?: InputMaybe<Scalars['String']['input']>;
  pagination?: InputMaybe<Pagination>;
  sort?: InputMaybe<AssetSortType>;
  workspaceId: Scalars['ID']['input'];
};


export type QueryDeploymentsArgs = {
  pagination?: InputMaybe<Pagination>;
  workspaceId: Scalars['ID']['input'];
};


export type QueryJobArgs = {
  id: Scalars['ID']['input'];
};


export type QueryJobsArgs = {
  pagination?: InputMaybe<Pagination>;
  workspaceId: Scalars['ID']['input'];
};


export type QueryNodeArgs = {
  id: Scalars['ID']['input'];
  type: NodeType;
};


export type QueryNodesArgs = {
  id: Array<Scalars['ID']['input']>;
  type: NodeType;
};


export type QueryProjectsArgs = {
  after?: InputMaybe<Scalars['Cursor']['input']>;
  before?: InputMaybe<Scalars['Cursor']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
  includeArchived?: InputMaybe<Scalars['Boolean']['input']>;
  last?: InputMaybe<Scalars['Int']['input']>;
  workspaceId: Scalars['ID']['input'];
};


export type QuerySearchUserArgs = {
  nameOrEmail: Scalars['String']['input'];
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
  projectId: Scalars['ID']['output'];
  started: Scalars['Boolean']['output'];
};

export type SignupInput = {
  secret?: InputMaybe<Scalars['String']['input']>;
  userId?: InputMaybe<Scalars['ID']['input']>;
  workspaceId?: InputMaybe<Scalars['ID']['input']>;
};

export type SignupPayload = {
  __typename?: 'SignupPayload';
  user: User;
  workspace: Workspace;
};

export type UpdateDeploymentInput = {
  deploymentId: Scalars['ID']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
  file?: InputMaybe<Scalars['Upload']['input']>;
};

export type UpdateMeInput = {
  email?: InputMaybe<Scalars['String']['input']>;
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

export type UpdateProjectInput = {
  archived?: InputMaybe<Scalars['Boolean']['input']>;
  basicAuthPassword?: InputMaybe<Scalars['String']['input']>;
  basicAuthUsername?: InputMaybe<Scalars['String']['input']>;
  description?: InputMaybe<Scalars['String']['input']>;
  isBasicAuthActive?: InputMaybe<Scalars['Boolean']['input']>;
  name?: InputMaybe<Scalars['String']['input']>;
  projectId: Scalars['ID']['input'];
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
  after?: InputMaybe<Scalars['Cursor']['input']>;
  before?: InputMaybe<Scalars['Cursor']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
  last?: InputMaybe<Scalars['Int']['input']>;
};


export type WorkspaceProjectsArgs = {
  after?: InputMaybe<Scalars['Cursor']['input']>;
  before?: InputMaybe<Scalars['Cursor']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
  includeArchived?: InputMaybe<Scalars['Boolean']['input']>;
  last?: InputMaybe<Scalars['Int']['input']>;
};

export type WorkspaceMember = {
  __typename?: 'WorkspaceMember';
  role: Role;
  user?: Maybe<User>;
  userId: Scalars['ID']['output'];
};

export type DeploymentFragment = { __typename?: 'Deployment', id: string, projectId: string, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null };

export type ProjectFragment = { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string };

export type JobFragment = { __typename?: 'Job', id: string, deploymentId: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null };

export type CreateDeploymentMutationVariables = Exact<{
  input: CreateDeploymentInput;
}>;


export type CreateDeploymentMutation = { __typename?: 'Mutation', createDeployment?: { __typename?: 'DeploymentPayload', deployment: { __typename?: 'Deployment', id: string, projectId: string, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } } | null };

export type UpdateDeploymentMutationVariables = Exact<{
  input: UpdateDeploymentInput;
}>;


export type UpdateDeploymentMutation = { __typename?: 'Mutation', updateDeployment?: { __typename?: 'DeploymentPayload', deployment: { __typename?: 'Deployment', id: string, projectId: string, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } } | null };

export type DeleteDeploymentMutationVariables = Exact<{
  input: DeleteDeploymentInput;
}>;


export type DeleteDeploymentMutation = { __typename?: 'Mutation', deleteDeployment?: { __typename?: 'DeleteDeploymentPayload', deploymentId: string } | null };

export type ExecuteDeploymentMutationVariables = Exact<{
  input: ExecuteDeploymentInput;
}>;


export type ExecuteDeploymentMutation = { __typename?: 'Mutation', executeDeployment?: { __typename?: 'JobPayload', job: { __typename?: 'Job', id: string, deploymentId: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null } } | null };

export type GetDeploymentsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  pagination?: InputMaybe<Pagination>;
}>;


export type GetDeploymentsQuery = { __typename?: 'Query', deployments: { __typename?: 'DeploymentConnection', totalCount: number, nodes: Array<{ __typename?: 'Deployment', id: string, projectId: string, workspaceId: string, workflowUrl: string, description: string, version: string, createdAt: any, updatedAt: any, project?: { __typename?: 'Project', name: string } | null } | null>, pageInfo: { __typename?: 'PageInfo', endCursor?: any | null, hasNextPage: boolean } } };

export type GetJobsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  pagination?: InputMaybe<Pagination>;
}>;


export type GetJobsQuery = { __typename?: 'Query', jobs: { __typename?: 'JobConnection', totalCount: number, nodes: Array<{ __typename?: 'Job', id: string, deploymentId: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null } | null>, pageInfo: { __typename?: 'PageInfo', endCursor?: any | null, hasNextPage: boolean } } };

export type GetJobQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetJobQuery = { __typename?: 'Query', job?: { __typename?: 'Job', id: string, deploymentId: string, workspaceId: string, status: JobStatus, startedAt: any, completedAt?: any | null } | null };

export type CreateProjectMutationVariables = Exact<{
  input: CreateProjectInput;
}>;


export type CreateProjectMutation = { __typename?: 'Mutation', createProject?: { __typename?: 'ProjectPayload', project: { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string } } | null };

export type GetProjectsQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
  first: Scalars['Int']['input'];
  after?: InputMaybe<Scalars['Cursor']['input']>;
}>;


export type GetProjectsQuery = { __typename?: 'Query', projects: { __typename?: 'ProjectConnection', totalCount: number, nodes: Array<{ __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string } | null>, pageInfo: { __typename?: 'PageInfo', endCursor?: any | null, hasNextPage: boolean } } };

export type GetProjectByIdQueryVariables = Exact<{
  projectId: Scalars['ID']['input'];
}>;


export type GetProjectByIdQuery = { __typename?: 'Query', node?: { __typename: 'Asset' } | { __typename: 'Deployment' } | { __typename: 'Job' } | { __typename: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string } | { __typename: 'User' } | { __typename: 'Workspace' } | null };

export type UpdateProjectMutationVariables = Exact<{
  input: UpdateProjectInput;
}>;


export type UpdateProjectMutation = { __typename?: 'Mutation', updateProject?: { __typename?: 'ProjectPayload', project: { __typename?: 'Project', id: string, name: string, description: string, createdAt: any, updatedAt: any, workspaceId: string } } | null };

export type DeleteProjectMutationVariables = Exact<{
  input: DeleteProjectInput;
}>;


export type DeleteProjectMutation = { __typename?: 'Mutation', deleteProject?: { __typename?: 'DeleteProjectPayload', projectId: string } | null };

export type RunProjectMutationVariables = Exact<{
  input: RunProjectInput;
}>;


export type RunProjectMutation = { __typename?: 'Mutation', runProject?: { __typename?: 'RunProjectPayload', projectId: string, started: boolean } | null };

export type GetMeQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMeQuery = { __typename?: 'Query', me?: { __typename?: 'Me', id: string, name: string, email: string, myWorkspaceId: string } | null };

export type SearchUserQueryVariables = Exact<{
  email: Scalars['String']['input'];
}>;


export type SearchUserQuery = { __typename?: 'Query', searchUser?: { __typename?: 'User', id: string, name: string, email: string } | null };

export type UpdateMeMutationVariables = Exact<{
  input: UpdateMeInput;
}>;


export type UpdateMeMutation = { __typename?: 'Mutation', updateMe?: { __typename?: 'UpdateMePayload', me: { __typename?: 'Me', id: string, name: string, email: string } } | null };

export type WorkspaceFragment = { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> };

export type CreateWorkspaceMutationVariables = Exact<{
  input: CreateWorkspaceInput;
}>;


export type CreateWorkspaceMutation = { __typename?: 'Mutation', createWorkspace?: { __typename?: 'CreateWorkspacePayload', workspace: { __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } } | null };

export type GetWorkspacesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetWorkspacesQuery = { __typename?: 'Query', me?: { __typename?: 'Me', id: string, workspaces: Array<{ __typename?: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> }> } | null };

export type GetWorkspaceByIdQueryVariables = Exact<{
  workspaceId: Scalars['ID']['input'];
}>;


export type GetWorkspaceByIdQuery = { __typename?: 'Query', node?: { __typename: 'Asset' } | { __typename: 'Deployment' } | { __typename: 'Job' } | { __typename: 'Project' } | { __typename: 'User' } | { __typename: 'Workspace', id: string, name: string, personal: boolean, members: Array<{ __typename?: 'WorkspaceMember', userId: string, role: Role, user?: { __typename?: 'User', id: string, email: string, name: string } | null }> } | null };

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
}
    `;
export const JobFragmentDoc = gql`
    fragment Job on Job {
  id
  deploymentId
  workspaceId
  status
  startedAt
  completedAt
}
    `;
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
    query GetDeployments($workspaceId: ID!, $pagination: Pagination) {
  deployments(workspaceId: $workspaceId, pagination: $pagination) {
    totalCount
    nodes {
      ...Deployment
    }
    pageInfo {
      endCursor
      hasNextPage
    }
  }
}
    ${DeploymentFragmentDoc}`;
export const GetJobsDocument = gql`
    query GetJobs($workspaceId: ID!, $pagination: Pagination) {
  jobs(workspaceId: $workspaceId, pagination: $pagination) {
    totalCount
    nodes {
      ...Job
    }
    pageInfo {
      endCursor
      hasNextPage
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
    query GetProjects($workspaceId: ID!, $first: Int!, $after: Cursor) {
  projects(workspaceId: $workspaceId, first: $first, after: $after) {
    totalCount
    nodes {
      ...Project
    }
    pageInfo {
      endCursor
      hasNextPage
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
    projectId
    started
  }
}
    `;
export const GetMeDocument = gql`
    query GetMe {
  me {
    id
    name
    email
    myWorkspaceId
  }
}
    `;
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
    GetJobs(variables: GetJobsQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetJobsQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetJobsQuery>(GetJobsDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetJobs', 'query', variables);
    },
    GetJob(variables: GetJobQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetJobQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetJobQuery>(GetJobDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetJob', 'query', variables);
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
    GetMe(variables?: GetMeQueryVariables, requestHeaders?: GraphQLClientRequestHeaders): Promise<GetMeQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetMeQuery>(GetMeDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'GetMe', 'query', variables);
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