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

export enum WorkspaceQueryKeys {
  GetWorkspace = "getWorkspace",
}

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
    queryKey: [WorkspaceQueryKeys.GetWorkspace],
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
