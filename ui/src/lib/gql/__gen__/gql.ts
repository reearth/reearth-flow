/* eslint-disable */
import * as types from './graphql';
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 */
const documents = {
    "fragment Project on Project {\n  id\n  name\n  description\n  createdAt\n  updatedAt\n  workspaceId\n}\n\nmutation CreateProject($input: CreateProjectInput!) {\n  createProject(input: $input) {\n    project {\n      ...Project\n    }\n  }\n}\n\nquery GetProjects($workspaceId: ID!, $first: Int!) {\n  projects(workspaceId: $workspaceId, first: $first) {\n    totalCount\n    nodes {\n      ...Project\n    }\n    pageInfo {\n      startCursor\n      endCursor\n      hasNextPage\n      hasPreviousPage\n    }\n  }\n}\n\nquery GetProjectById($projectId: ID!) {\n  node(id: $projectId, type: PROJECT) {\n    __typename\n    ...Project\n  }\n}\n\nmutation UpdateProject($input: UpdateProjectInput!) {\n  updateProject(input: $input) {\n    project {\n      ...Project\n    }\n  }\n}\n\nmutation DeleteProject($input: DeleteProjectInput!) {\n  deleteProject(input: $input) {\n    projectId\n  }\n}": types.ProjectFragmentDoc,
    "query GetMe {\n  me {\n    id\n    name\n    email\n    myWorkspaceId\n  }\n}\n\nquery SearchUser($email: String!) {\n  searchUser(nameOrEmail: $email) {\n    id\n    name\n    email\n  }\n}": types.GetMeDocument,
    "fragment Workspace on Workspace {\n  id\n  name\n  personal\n  members {\n    userId\n    role\n    user {\n      id\n      email\n      name\n    }\n  }\n}\n\nmutation CreateWorkspace($input: CreateWorkspaceInput!) {\n  createWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nquery GetWorkspaces {\n  me {\n    workspaces {\n      ...Workspace\n    }\n  }\n}\n\nquery GetWorkspaceById($workspaceId: ID!) {\n  node(id: $workspaceId, type: WORKSPACE) {\n    __typename\n    ...Workspace\n  }\n}\n\nmutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n  updateWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation DeleteWorkspace($input: DeleteWorkspaceInput!) {\n  deleteWorkspace(input: $input) {\n    workspaceId\n  }\n}\n\nmutation AddMemberToWorkspace($input: AddMemberToWorkspaceInput!) {\n  addMemberToWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation RemoveMemberFromWorkspace($input: RemoveMemberFromWorkspaceInput!) {\n  removeMemberFromWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation UpdateMemberOfWorkspace($input: UpdateMemberOfWorkspaceInput!) {\n  updateMemberOfWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}": types.WorkspaceFragmentDoc,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = graphql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function graphql(source: string): unknown;

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "fragment Project on Project {\n  id\n  name\n  description\n  createdAt\n  updatedAt\n  workspaceId\n}\n\nmutation CreateProject($input: CreateProjectInput!) {\n  createProject(input: $input) {\n    project {\n      ...Project\n    }\n  }\n}\n\nquery GetProjects($workspaceId: ID!, $first: Int!) {\n  projects(workspaceId: $workspaceId, first: $first) {\n    totalCount\n    nodes {\n      ...Project\n    }\n    pageInfo {\n      startCursor\n      endCursor\n      hasNextPage\n      hasPreviousPage\n    }\n  }\n}\n\nquery GetProjectById($projectId: ID!) {\n  node(id: $projectId, type: PROJECT) {\n    __typename\n    ...Project\n  }\n}\n\nmutation UpdateProject($input: UpdateProjectInput!) {\n  updateProject(input: $input) {\n    project {\n      ...Project\n    }\n  }\n}\n\nmutation DeleteProject($input: DeleteProjectInput!) {\n  deleteProject(input: $input) {\n    projectId\n  }\n}"): (typeof documents)["fragment Project on Project {\n  id\n  name\n  description\n  createdAt\n  updatedAt\n  workspaceId\n}\n\nmutation CreateProject($input: CreateProjectInput!) {\n  createProject(input: $input) {\n    project {\n      ...Project\n    }\n  }\n}\n\nquery GetProjects($workspaceId: ID!, $first: Int!) {\n  projects(workspaceId: $workspaceId, first: $first) {\n    totalCount\n    nodes {\n      ...Project\n    }\n    pageInfo {\n      startCursor\n      endCursor\n      hasNextPage\n      hasPreviousPage\n    }\n  }\n}\n\nquery GetProjectById($projectId: ID!) {\n  node(id: $projectId, type: PROJECT) {\n    __typename\n    ...Project\n  }\n}\n\nmutation UpdateProject($input: UpdateProjectInput!) {\n  updateProject(input: $input) {\n    project {\n      ...Project\n    }\n  }\n}\n\nmutation DeleteProject($input: DeleteProjectInput!) {\n  deleteProject(input: $input) {\n    projectId\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "query GetMe {\n  me {\n    id\n    name\n    email\n    myWorkspaceId\n  }\n}\n\nquery SearchUser($email: String!) {\n  searchUser(nameOrEmail: $email) {\n    id\n    name\n    email\n  }\n}"): (typeof documents)["query GetMe {\n  me {\n    id\n    name\n    email\n    myWorkspaceId\n  }\n}\n\nquery SearchUser($email: String!) {\n  searchUser(nameOrEmail: $email) {\n    id\n    name\n    email\n  }\n}"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "fragment Workspace on Workspace {\n  id\n  name\n  personal\n  members {\n    userId\n    role\n    user {\n      id\n      email\n      name\n    }\n  }\n}\n\nmutation CreateWorkspace($input: CreateWorkspaceInput!) {\n  createWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nquery GetWorkspaces {\n  me {\n    workspaces {\n      ...Workspace\n    }\n  }\n}\n\nquery GetWorkspaceById($workspaceId: ID!) {\n  node(id: $workspaceId, type: WORKSPACE) {\n    __typename\n    ...Workspace\n  }\n}\n\nmutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n  updateWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation DeleteWorkspace($input: DeleteWorkspaceInput!) {\n  deleteWorkspace(input: $input) {\n    workspaceId\n  }\n}\n\nmutation AddMemberToWorkspace($input: AddMemberToWorkspaceInput!) {\n  addMemberToWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation RemoveMemberFromWorkspace($input: RemoveMemberFromWorkspaceInput!) {\n  removeMemberFromWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation UpdateMemberOfWorkspace($input: UpdateMemberOfWorkspaceInput!) {\n  updateMemberOfWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}"): (typeof documents)["fragment Workspace on Workspace {\n  id\n  name\n  personal\n  members {\n    userId\n    role\n    user {\n      id\n      email\n      name\n    }\n  }\n}\n\nmutation CreateWorkspace($input: CreateWorkspaceInput!) {\n  createWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nquery GetWorkspaces {\n  me {\n    workspaces {\n      ...Workspace\n    }\n  }\n}\n\nquery GetWorkspaceById($workspaceId: ID!) {\n  node(id: $workspaceId, type: WORKSPACE) {\n    __typename\n    ...Workspace\n  }\n}\n\nmutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n  updateWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation DeleteWorkspace($input: DeleteWorkspaceInput!) {\n  deleteWorkspace(input: $input) {\n    workspaceId\n  }\n}\n\nmutation AddMemberToWorkspace($input: AddMemberToWorkspaceInput!) {\n  addMemberToWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation RemoveMemberFromWorkspace($input: RemoveMemberFromWorkspaceInput!) {\n  removeMemberFromWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}\n\nmutation UpdateMemberOfWorkspace($input: UpdateMemberOfWorkspaceInput!) {\n  updateMemberOfWorkspace(input: $input) {\n    workspace {\n      ...Workspace\n    }\n  }\n}"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;