import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

import { CreateWorkspaceMutation } from "../__gen__/graphql";

export enum WorkspaceQueryKeys {
  GetWorkspace = "getWorkspace",
}

type mutationInput = {
  onSuccess?: () => void;
  onError?: () => void;
};

export const useCreateWorkspaceMutation = ({
  onSuccess,
  onError,
}: {
  onSuccess: (data: CreateWorkspaceMutation) => void;
  onError: () => void;
}) => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: graphQLContext?.CreateWorkspace,
    onError: onError,
    onSuccess: data => {
      queryClient.invalidateQueries({ queryKey: [WorkspaceQueryKeys.GetWorkspace] });
      onSuccess && onSuccess(data);
    },
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

export const useUpdateWorkspaceMutation = ({ onSuccess, onError }: mutationInput) => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: graphQLContext?.UpdateWorkspace,
    onError: onError,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [WorkspaceQueryKeys.GetWorkspace] });
      onSuccess && onSuccess();
    },
  });
};

export const useDeleteWorkspaceQuery = ({ onSuccess, onError }: mutationInput) => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: graphQLContext?.DeleteWorkspace,
    onError: onError,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [WorkspaceQueryKeys.GetWorkspace] });
      onSuccess && onSuccess();
    },
  });
};
