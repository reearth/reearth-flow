import { useMutation, useQuery } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

import { graphql } from "../__gen__";

graphql(`
  mutation CreateWorkspace($input: CreateWorkspaceInput!) {
    createWorkspace(input: $input) {
      workspace {
        name
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

// TODO: add onSuccess and onError types
export const useCreateWorkspaceMutation = ({ onSuccess, onError }) => {
  const graphQLContext = useGraphQLContext();
  return useMutation({
    mutationFn: graphQLContext?.CreateWorkspace,
    onSuccess: onSuccess,
    onError: onError,
    // TODO: use the function below to invalidate the query
    // onSuccess: () => {
    //   queryClient.invalidateQueries({ queryKey: ['getWorkspace'] })
    // },
  });
};

export const useGetWorkspaceQuery = () => {
  const graphQLContext = useGraphQLContext();
  const { data, ...rest } = useQuery({
    queryKey: ["getWorkspace"],
    queryFn: async () => graphQLContext?.GetWorkspaces(),
  });

  return { data, ...rest };
};

export const useUpdateWorkspaceMutation = () => {
  const graphQLContext = useGraphQLContext();
  return useMutation({
    mutationFn: graphQLContext?.UpdateWorkspace,
  });
};

export const useDeleteWorkspaceQuery = () => {
  const graphQLContext = useGraphQLContext();

  return useMutation({
    mutationFn: graphQLContext?.DeleteWorkspace,
  });
};
