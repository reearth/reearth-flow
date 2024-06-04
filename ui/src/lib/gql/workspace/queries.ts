import { graphql } from "../__gen__";

graphql(`
  fragment GetWorkspace on Workspace {
    id
    name
    personal
  }
`);

graphql(`
  mutation CreateWorkspace($input: CreateWorkspaceInput!) {
    createWorkspace(input: $input) {
      workspace {
        ...GetWorkspace
      }
    }
  }
`);

// TODO: Should this a fragment in GET_ME?
graphql(`
  query GetWorkspaces {
    me {
      workspaces {
        ...GetWorkspace
      }
    }
  }
`);

graphql(`
  mutation UpdateWorkspace($input: UpdateWorkspaceInput!) {
    updateWorkspace(input: $input) {
      workspace {
        ...GetWorkspace
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
