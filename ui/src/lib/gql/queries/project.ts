import { graphql } from "@flow/lib/gql";

export const CREATE_PROJECT = graphql(`
  mutation CreateProject($input: CreateProjectInput!) {
    createProject(input: $input) {
      project {
        id
      }
    }
  }
`);

export const GET_PROJECTS = graphql(`
  query GetProjects($workspaceId: ID!, $first: Int!) {
    projects(workspaceId: $workspaceId, first: $first) {
      edges {
        node {
          id
          name
        }
      }
    }
  }
`);

export const UPDATE_PROJECT = graphql(`
  mutation UpdateProject($input: UpdateProjectInput!) {
    updateProject(input: $input) {
      project {
        id
        name
      }
    }
  }
`);

export const DELETE_PROJECT = graphql(`
  mutation DeleteProject($input: DeleteProjectInput!) {
    deleteProject(input: $input) {
      projectId
    }
  }
`);
