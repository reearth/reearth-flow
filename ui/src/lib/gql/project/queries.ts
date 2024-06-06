import { graphql } from "../__gen__";

graphql(`
  fragment Project on Project {
    name
    id
    description
    createdAt
    updatedAt
    isArchived
    workspaceId
  }
`);

graphql(`
  mutation CreateProject($input: CreateProjectInput!) {
    createProject(input: $input) {
      project {
        ...Project
      }
    }
  }
`);

graphql(`
  query GetProjects($workspaceId: ID!, $first: Int!) {
    projects(workspaceId: $workspaceId, first: $first) {
      totalCount
      nodes {
        ...Project
      }
      pageInfo {
        startCursor
        endCursor
        hasNextPage
        hasPreviousPage
      }
    }
  }
`);

graphql(`
  mutation UpdateProject($input: UpdateProjectInput!) {
    updateProject(input: $input) {
      project {
        ...Project
      }
    }
  }
`);

graphql(`
  mutation DeleteProject($input: DeleteProjectInput!) {
    deleteProject(input: $input) {
      projectId
    }
  }
`);
