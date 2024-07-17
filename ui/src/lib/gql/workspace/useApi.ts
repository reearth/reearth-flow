import { GetWorkspaces, DeleteWorkspace, GetWorkspace, Role, WorkspaceMutation } from "@flow/types";

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
    updateWorkspaceMutation,
    addMemberToWorkspaceMutation,
    removeMemberFromWorkspaceMutation,
    updateMemberOfWorkspaceMutation,
  } = useQueries();

  const createWorkspace = async (name: string): Promise<WorkspaceMutation> => {
    const { mutateAsync, ...rest } = createWorkspaceMutation;
    try {
      const data = await mutateAsync(name);
      return { workspace: data, ...rest };
    } catch (err) {
      return { workspace: undefined, ...rest };
    }
  };

  const useGetWorkspaces = (): GetWorkspaces => {
    const { data: workspaces, ...rest } = useGetWorkspacesQuery();
    return {
      workspaces,
      ...rest,
    };
  };

  const useGetWorkspace = (workspaceId?: string): GetWorkspace => {
    const { data: workspace, ...rest } = useGetWorkspaceByIdQuery(workspaceId);
    return {
      workspace,
      ...rest,
    };
  };

  const updateWorkspace = async (workspaceId: string, name: string): Promise<WorkspaceMutation> => {
    const { mutateAsync, ...rest } = updateWorkspaceMutation;
    try {
      const data = await mutateAsync({ workspaceId, name });
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

  // Members in the Workspace
  const addMemberToWorkspace = async (
    workspaceId: string,
    userId: string,
    role: Role,
  ): Promise<WorkspaceMutation> => {
    const { mutateAsync, ...rest } = addMemberToWorkspaceMutation;
    try {
      const data = await mutateAsync({ workspaceId, userId, role });
      return { workspace: data, ...rest };
    } catch (err) {
      return { workspace: undefined, ...rest };
    }
  };

  const removeMemberFromWorkspace = async (
    workspaceId: string,
    userId: string,
  ): Promise<WorkspaceMutation> => {
    const { mutateAsync, ...rest } = removeMemberFromWorkspaceMutation;
    try {
      const data = await mutateAsync({ workspaceId, userId });
      return { workspace: data, ...rest };
    } catch (err) {
      return { workspace: undefined, ...rest };
    }
  };

  const updateMemberOfWorkspace = async (
    workspaceId: string,
    userId: string,
    role: Role,
  ): Promise<WorkspaceMutation> => {
    const { mutateAsync, ...rest } = updateMemberOfWorkspaceMutation;
    try {
      const data = await mutateAsync({ workspaceId, userId, role });
      return { workspace: data, ...rest };
    } catch (err) {
      return { workspace: undefined, ...rest };
    }
  };

  return {
    createWorkspace,
    useGetWorkspaces,
    useGetWorkspace,
    deleteWorkspace,
    updateWorkspace,
    addMemberToWorkspace,
    removeMemberFromWorkspace,
    updateMemberOfWorkspace,
  };
};
