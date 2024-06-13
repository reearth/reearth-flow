import { GetWorkspaces, CreateWorkspace, DeleteWorkspace, GetWorkspace } from "@flow/types";

import { useQueries } from "./useQueries";

export enum WorkspaceQueryKeys {
  GetWorkspaces = "getWorkspaces",
  GetWorkspace = "getWorkspace",
}

export const useWorkspace = () => {
  const {
    createWorkspaceMutation,
    useGetWorkspacesQuery,
    deleteWorkspaceMutation,
    useGetWorkspaceByIdQuery,
  } = useQueries();

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

  const useGetWorkspaces = (): GetWorkspaces => {
    const { data: workspaces, ...rest } = useGetWorkspacesQuery();
    return {
      workspaces,
      ...rest,
    };
  };

  const useGetWorkspace = (workspaceId: string): GetWorkspace => {
    const { data: workspace, ...rest } = useGetWorkspaceByIdQuery(workspaceId);
    return {
      workspace,
      ...rest,
    };
  };

  return {
    createWorkspace,
    useGetWorkspaces,
    useGetWorkspace,
    deleteWorkspace,
  };
};
