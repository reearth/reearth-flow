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
    "\n  query GetMe {\n    me {\n      id\n      name\n      email\n      myWorkspaceId\n    }\n  }\n": types.GetMeDocument,
    "\n  mutation CreateWorkspace($input: CreateWorkspaceInput!) {\n    createWorkspace(input: $input) {\n      workspace {\n        name\n      }\n    }\n  }\n": types.CreateWorkspaceDocument,
    "\n  query GetWorkspaces {\n    me {\n      workspaces {\n        name\n        id\n        personal\n      }\n    }\n  }\n": types.GetWorkspacesDocument,
    "\n  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n    updateWorkspace(input: $input) {\n      workspace {\n        id\n        name\n      }\n    }\n  }\n": types.UpdateWorkspaceDocument,
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
export function graphql(source: "\n  query GetMe {\n    me {\n      id\n      name\n      email\n      myWorkspaceId\n    }\n  }\n"): (typeof documents)["\n  query GetMe {\n    me {\n      id\n      name\n      email\n      myWorkspaceId\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CreateWorkspace($input: CreateWorkspaceInput!) {\n    createWorkspace(input: $input) {\n      workspace {\n        name\n      }\n    }\n  }\n"): (typeof documents)["\n  mutation CreateWorkspace($input: CreateWorkspaceInput!) {\n    createWorkspace(input: $input) {\n      workspace {\n        name\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query GetWorkspaces {\n    me {\n      workspaces {\n        name\n        id\n        personal\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetWorkspaces {\n    me {\n      workspaces {\n        name\n        id\n        personal\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n    updateWorkspace(input: $input) {\n      workspace {\n        id\n        name\n      }\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {\n    updateWorkspace(input: $input) {\n      workspace {\n        id\n        name\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation DeleteWorkspace($input: DeleteWorkspaceInput!) {\n    deleteWorkspace(input: $input) {\n      workspaceId\n    }\n  }\n"): (typeof documents)["\n  mutation DeleteWorkspace($input: DeleteWorkspaceInput!) {\n    deleteWorkspace(input: $input) {\n      workspaceId\n    }\n  }\n"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;