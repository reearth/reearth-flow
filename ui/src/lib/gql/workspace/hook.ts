import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { Workspace, GetWorkspace, CreateWorkspace, DeleteWorkspace } from "@flow/types";

export enum WorkspaceQueryKeys {
  GetWorkspace = "getWorkspace",
}

export const useWorkspace = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createWorkspaceMutation = useMutation({
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

  const getWorkspacesQuery = useQuery({
    queryKey: [WorkspaceQueryKeys.GetWorkspace],
    queryFn: async () => {
      if (!graphQLContext?.GetWorkspaces) return;
      const data = await graphQLContext.GetWorkspaces();
      return data?.me?.workspaces;
    },
    staleTime: Infinity,
  });

  const deleteWorkspaceMutation = useMutation({
    mutationFn: async (workspaceId: string) => {
      const data = await graphQLContext?.DeleteWorkspace({ input: { workspaceId } });
      return data?.deleteWorkspace?.workspaceId;
    },
    onSuccess: deletedWorkspaceId => {
      queryClient.setQueryData([WorkspaceQueryKeys.GetWorkspace], (data: Workspace[]) => {
        data.splice(
          data.findIndex(w => w.id === deletedWorkspaceId),
          1,
        );
        return [...data];
      });
    },
  });

  const createWorkspace = async (name: string): Promise<CreateWorkspace> => {
    const { mutateAsync, ...rest } = createWorkspaceMutation;
    try {
      const data = await mutateAsync(name);
      return { workspace: data, ...rest };
    } catch (err) {
      return { workspace: undefined, ...rest };
    }
  };
  const deleteWorkspace = async (workspaceId: string): Promise<DeleteWorkspace> => {
    const { mutateAsync, ...rest } = deleteWorkspaceMutation;
    try {
      const data = await mutateAsync(workspaceId);
      return { workspaceId: data, ...rest };
    } catch (err) {
      return { workspaceId: undefined, ...rest };
    }
  };

  const getWorkspaces = (): GetWorkspace => {
    const { data: workspaces, ...rest } = getWorkspacesQuery;
    return {
      workspaces,
      ...rest,
    };
  };

  return {
    createWorkspace,
    getWorkspaces,
    deleteWorkspace,
  };
};
