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
    "\n  fragment Project on Project {\n    name\n    id\n    description\n    createdAt\n    updatedAt\n    isArchived\n    workspaceId\n  }\n": types.ProjectFragmentDoc,
    "\n  mutation CreateProject($input: CreateProjectInput!) {\n    createProject(input: $input) {\n      project {\n        ...Project\n      }\n    }\n  }\n": types.CreateProjectDocument,
    "\n  query GetProjects($workspaceId: ID!, $first: Int!) {\n    projects(workspaceId: $workspaceId, first: $first) {\n      totalCount\n      nodes {\n        ...Project\n      }\n      pageInfo {\n        startCursor\n        endCursor\n        hasNextPage\n        hasPreviousPage\n      }\n    }\n  }\n": types.GetProjectsDocument,
    "\n  mutation UpdateProject($input: UpdateProjectInput!) {\n    updateProject(input: $input) {\n      project {\n        ...Project\n      }\n    }\n  }\n": types.UpdateProjectDocument,
    "\n  mutation DeleteProject($input: DeleteProjectInput!) {\n    deleteProject(input: $input) {\n      projectId\n    }\n  }\n": types.DeleteProjectDocument,
    "\n  query GetMe {\n    me {\n      id\n      name\n      email\n      myWorkspaceId\n    }\n  }\n": types.GetMeDocument,
    "\n  fragment GetWorkspace on Workspace {\n    id\n    name\n    personal\n  }\n": types.GetWorkspaceFragmentDoc,
    "\n  mutation CreateWorkspace($input: CreateWorkspaceInput!) {\n    createWorkspace(input: $input) {\n      workspace {\n        ...GetWorkspace\n      }\n    }\n  }\n": types.CreateWorkspaceDocument,
    "\n  query GetWorkspaces {\n    me {\n      workspaces {\n        ...GetWorkspace\n      }\n    }\n  }\n": types.GetWorkspacesDocument,
    "\n  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n    updateWorkspace(input: $input) {\n      workspace {\n        ...GetWorkspace\n      }\n    }\n  }\n": types.UpdateWorkspaceDocument,
    "\n  mutation DeleteWorkspace($input: DeleteWorkspaceInput!) {\n    deleteWorkspace(input: $input) {\n      workspaceId\n    }\n  }\n": types.DeleteWorkspaceDocument,
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
export function graphql(source: "\n  fragment Project on Project {\n    name\n    id\n    description\n    createdAt\n    updatedAt\n    isArchived\n    workspaceId\n  }\n"): (typeof documents)["\n  fragment Project on Project {\n    name\n    id\n    description\n    createdAt\n    updatedAt\n    isArchived\n    workspaceId\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CreateProject($input: CreateProjectInput!) {\n    createProject(input: $input) {\n      project {\n        ...Project\n      }\n    }\n  }\n"): (typeof documents)["\n  mutation CreateProject($input: CreateProjectInput!) {\n    createProject(input: $input) {\n      project {\n        ...Project\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query GetProjects($workspaceId: ID!, $first: Int!) {\n    projects(workspaceId: $workspaceId, first: $first) {\n      totalCount\n      nodes {\n        ...Project\n      }\n      pageInfo {\n        startCursor\n        endCursor\n        hasNextPage\n        hasPreviousPage\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetProjects($workspaceId: ID!, $first: Int!) {\n    projects(workspaceId: $workspaceId, first: $first) {\n      totalCount\n      nodes {\n        ...Project\n      }\n      pageInfo {\n        startCursor\n        endCursor\n        hasNextPage\n        hasPreviousPage\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation UpdateProject($input: UpdateProjectInput!) {\n    updateProject(input: $input) {\n      project {\n        ...Project\n      }\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateProject($input: UpdateProjectInput!) {\n    updateProject(input: $input) {\n      project {\n        ...Project\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation DeleteProject($input: DeleteProjectInput!) {\n    deleteProject(input: $input) {\n      projectId\n    }\n  }\n"): (typeof documents)["\n  mutation DeleteProject($input: DeleteProjectInput!) {\n    deleteProject(input: $input) {\n      projectId\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query GetMe {\n    me {\n      id\n      name\n      email\n      myWorkspaceId\n    }\n  }\n"): (typeof documents)["\n  query GetMe {\n    me {\n      id\n      name\n      email\n      myWorkspaceId\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment GetWorkspace on Workspace {\n    id\n    name\n    personal\n  }\n"): (typeof documents)["\n  fragment GetWorkspace on Workspace {\n    id\n    name\n    personal\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CreateWorkspace($input: CreateWorkspaceInput!) {\n    createWorkspace(input: $input) {\n      workspace {\n        ...GetWorkspace\n      }\n    }\n  }\n"): (typeof documents)["\n  mutation CreateWorkspace($input: CreateWorkspaceInput!) {\n    createWorkspace(input: $input) {\n      workspace {\n        ...GetWorkspace\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query GetWorkspaces {\n    me {\n      workspaces {\n        ...GetWorkspace\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetWorkspaces {\n    me {\n      workspaces {\n        ...GetWorkspace\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n    updateWorkspace(input: $input) {\n      workspace {\n        ...GetWorkspace\n      }\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n    updateWorkspace(input: $input) {\n      workspace {\n        ...GetWorkspace\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation DeleteWorkspace($input: DeleteWorkspaceInput!) {\n    deleteWorkspace(input: $input) {\n      workspaceId\n    }\n  }\n"): (typeof documents)["\n  mutation DeleteWorkspace($input: DeleteWorkspaceInput!) {\n    deleteWorkspace(input: $input) {\n      workspaceId\n    }\n  }\n"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;