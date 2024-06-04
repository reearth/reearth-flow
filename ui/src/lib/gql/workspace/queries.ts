import { graphql } from "../__gen__";

graphql(`
  mutation CreateWorkspace($input: CreateWorkspaceInput!) {
    createWorkspace(input: $input) {
      workspace {
        id
      }
    }
  }
`);

// TODO: Should this a fragment in GET_ME?
graphql(`
  query GetWorkspaces {
    me {
      workspaces {
        id
        name
        personal
      }
    }
  }
`);

graphql(`
  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {
    updateWorkspace(input: $input) {
      workspace {
        id
        name
      }
    }
  }
`);

graphql(`
  mutation DeleteWorkspace($input: DeleteWorkspaceInput!) {
    deleteWorkspace(input: $input) {
      workspaceId
    }
  }
`);
