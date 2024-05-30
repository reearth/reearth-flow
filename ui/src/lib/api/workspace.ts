import { useMutation, useQuery } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

// TODO: const graphQLContext = useGraphQLContext(); is repeated everywhere
// graphQLContext?.[ACTION]({ input }) is also repeated everywhere

// TODO: Wite type definition for onSuccess and onError
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
