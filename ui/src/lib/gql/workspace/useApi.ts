import { GetWorkspace, CreateWorkspace, DeleteWorkspace } from "@flow/types";

import { useQueries } from "./useQueries";

export enum WorkspaceQueryKeys {
  GetWorkspace = "getWorkspace",
}

export const useWorkspace = () => {
  const { createWorkspaceMutation, getWorkspacesQuery, deleteWorkspaceMutation } = useQueries();

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
