import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { Workspace, useGraphQLContext } from "@flow/lib/gql";

export enum WorkspaceQueryKeys {
  GetWorkspace = "getWorkspace",
}

type CreateWorkspace = {
  createWorkspace: (name: string) => Promise<Workspace>;
  data: Workspace | undefined;
  isError: boolean;
  isSuccess: boolean;
  isPending: boolean;
  error: unknown;
};

export const useCreateWorkspaceMutation = (): CreateWorkspace => {
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
    createWorkspace: mutateAsync as (name: string) => Promise<Workspace>,
    data,
    ...rest,
  };
};

type GetWorkspace = {
  workspaces: Workspace[] | undefined;
  // TODO: These are generic so use declare them only once
  isLoading: boolean;
  isError: boolean;
  isSuccess: boolean;
  isPending: boolean;
  error: unknown;
};

export const useGetWorkspaceQuery = (): GetWorkspace => {
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
