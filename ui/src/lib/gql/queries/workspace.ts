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
        name
        id
        personal
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
