import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

export enum WorkspaceQueryKeys {
  GetWorkspace = "getWorkspace",
}

export const useCreateWorkspaceMutation = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: graphQLContext?.CreateWorkspace,
    onSuccess: async () =>
      await queryClient.invalidateQueries({ queryKey: [WorkspaceQueryKeys.GetWorkspace] }),
  });
};

export const useGetWorkspaceQuery = () => {
  const graphQLContext = useGraphQLContext();

  const { data, ...rest } = useQuery({
    queryKey: [WorkspaceQueryKeys.GetWorkspace],
    queryFn: async () => await graphQLContext?.GetWorkspaces(),
  });

  return { data, ...rest };
};

export const useUpdateWorkspaceMutation = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: graphQLContext?.UpdateWorkspace,
    onSuccess: async () =>
      await queryClient.invalidateQueries({ queryKey: [WorkspaceQueryKeys.GetWorkspace] }),
  });
};

export const useDeleteWorkspaceQuery = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: graphQLContext?.DeleteWorkspace,
    onSuccess: async () =>
      await queryClient.invalidateQueries({ queryKey: [WorkspaceQueryKeys.GetWorkspace] }),
  });
};
