import { graphql } from "@flow/lib/gql";

export const CREATE_WORKSPACE = graphql(`
  mutation CreateWorkspace($input: CreateWorkspaceInput!) {
    createWorkspace(input: $input) {
      workspace {
        name
      }
    }
  }
`);

// TODO: Should this a fragment in GET_ME?
export const GET_WORSPACES = graphql(`
  query GetWorkspaces {
    me {
      workspaces {
        id
        name
        members {
          userId
        }
        personal
        assets(first: 5) {
          nodes {
            id
          }
          edges {
            cursor
          }
          totalCount
          pageInfo {
            startCursor
            endCursor
            hasNextPage
            hasPreviousPage
          }
        }
        projects(first: 5) {
          nodes {
            id
          }
        }
      }
    }
  }
`);

export const UPDATE_WORKSPACE = graphql(`
  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {
    updateWorkspace(input: $input) {
      workspace {
        id
        name
      }
    }
  }
`);

export const DELETE_WORKSPACE = graphql(`
  mutation DeleteWorkspace($input: DeleteWorkspaceInput!) {
    deleteWorkspace(input: $input) {
      workspaceId
    }
  }
`);
