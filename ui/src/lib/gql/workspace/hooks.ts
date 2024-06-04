import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { Workspace } from "@flow/types";

export enum WorkspaceQueryKeys {
  GetWorkspace = "getWorkspace",
}

// TODO: This needs to be derived from the tanstack
type CommonReturnType = {
  isError: boolean;
  isSuccess: boolean;
  isPending: boolean;
  error: unknown;
};

type CreateWorkspace = {
  createWorkspace: (name: string) => Promise<Workspace | undefined>;
  data: Workspace | undefined;
} & CommonReturnType;

export const useCreateWorkspace = (): CreateWorkspace => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();
  const { data, mutateAsync, ...rest } = useMutation({
    mutationFn: async (name: string) => {
      const data = await graphQLContext?.CreateWorkspace({ input: { name } });
      return data?.createWorkspace?.workspace;
    },
    onSuccess: createdWorkspace => {
      queryClient.setQueryData([WorkspaceQueryKeys.GetWorkspace], (data: Workspace[]) => [
        ...data,
        createdWorkspace,
      ]);
    },
  });
  return {
    createWorkspace: mutateAsync,
    data,
    ...rest,
  };
};

type GetWorkspace = {
  workspaces: Workspace[] | undefined;
  isLoading: boolean;
} & CommonReturnType;

export const useGetWorkspace = (): GetWorkspace => {
  const graphQLContext = useGraphQLContext();

  const { data, ...rest } = useQuery({
    queryKey: [WorkspaceQueryKeys.GetWorkspace],
    queryFn: async () => {
      if (!graphQLContext?.GetWorkspaces) return;
      const data = await graphQLContext.GetWorkspaces();
      return data?.me?.workspaces;
    },
    staleTime: Infinity,
  });

  return { workspaces: data, ...rest };
};
