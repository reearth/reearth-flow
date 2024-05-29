import { GraphQLClient } from 'graphql-request';
import { RequestInit } from 'graphql-request/dist/types.dom';
import { useQuery, UseQueryOptions } from '@tanstack/react-query';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };

function fetcher<TData, TVariables extends { [key: string]: any }>(client: GraphQLClient, query: string, variables?: TVariables, requestHeaders?: RequestInit['headers']) {
  return async (): Promise<TData> => client.request({
    document: query,
    variables,
    requestHeaders
  });
}
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
  createProject?: Maybe<ProjectPayload>;
  createWorkspace?: Maybe<CreateWorkspacePayload>;
  deleteMe?: Maybe<DeleteMePayload>;
  deleteProject?: Maybe<DeleteProjectPayload>;
  deleteWorkspace?: Maybe<DeleteWorkspacePayload>;
  removeAsset?: Maybe<RemoveAssetPayload>;
  removeMemberFromWorkspace?: Maybe<RemoveMemberFromWorkspacePayload>;
  removeMyAuth?: Maybe<UpdateMePayload>;
  signup?: Maybe<SignupPayload>;
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


export type MutationCreateProjectArgs = {
  input: CreateProjectInput;
};


export type MutationCreateWorkspaceArgs = {
  input: CreateWorkspaceInput;
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


export type MutationRemoveAssetArgs = {
  input: RemoveAssetInput;
};


export type MutationRemoveMemberFromWorkspaceArgs = {
  input: RemoveMemberFromWorkspaceInput;
};


export type MutationRemoveMyAuthArgs = {
  input: RemoveMyAuthInput;
};


export type MutationSignupArgs = {
  input: SignupInput;
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

export type GetMeQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMeQuery = { __typename?: 'Query', me?: { __typename?: 'Me', id: string, name: string, email: string } | null };



export const GetMeDocument = `
    query GetMe {
  me {
    id
    name
    email
  }
}
    `;

export const useGetMeQuery = <
      TData = GetMeQuery,
      TError = unknown
    >(
      client: GraphQLClient,
      variables?: GetMeQueryVariables,
      options?: UseQueryOptions<GetMeQuery, TError, TData>,
      headers?: RequestInit['headers']
    ) => {
    
    return useQuery<GetMeQuery, TError, TData>(
      variables === undefined ? ['GetMe'] : ['GetMe', variables],
      fetcher<GetMeQuery, GetMeQueryVariables>(client, GetMeDocument, variables, headers),
      options
    )};
