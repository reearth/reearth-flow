import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import {
  GetWorkspaces,
  DeleteWorkspace,
  GetWorkspace,
  Role,
  WorkspaceMutation,
} from "@flow/types";

import { useQueries } from "./useQueries";

export enum WorkspaceQueryKeys {
  GetWorkspaces = "getWorkspaces",
  GetWorkspace = "getWorkspace",
}

export const useWorkspace = () => {
  const { toast } = useToast();
  const t = useT();

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
      toast({
        title: t("Workspace Created"),
        description: t("Workspace has been successfully created."),
      });
      return { workspace: data, ...rest };
    } catch (_err) {
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

  const updateWorkspace = async (
    workspaceId: string,
    name: string,
  ): Promise<WorkspaceMutation> => {
    const { mutateAsync, ...rest } = updateWorkspaceMutation;
    try {
      const data = await mutateAsync({ workspaceId, name });
      return { workspace: data, ...rest };
    } catch (_err) {
      return { workspace: undefined, ...rest };
    }
  };

  const deleteWorkspace = async (
    workspaceId: string,
  ): Promise<DeleteWorkspace> => {
    const { mutateAsync, ...rest } = deleteWorkspaceMutation;
    try {
      const data = await mutateAsync(workspaceId);
      toast({
        title: t("Workspace Deleted"),
        description: t("Workspace has been successfully deleted."),
        variant: "destructive",
      });
      return { workspaceId: data, ...rest };
    } catch (_err) {
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
      toast({
        title: t("Member Added"),
        description: t("Member has been successfully added to the workspace."),
      });
      return { workspace: data, ...rest };
    } catch (_err) {
      toast({
        title: t("Member Could Not Be Added"),
        description: t("There was an error when adding a new member"),
        variant: "warning",
      });
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
      toast({
        title: t("Member Removed"),
        description: t(
          "Member has been successfully removed from the workspace.",
        ),
        variant: "destructive",
      });
      return { workspace: data, ...rest };
    } catch (_err) {
      toast({
        title: t("Member Could Not Be Removed"),
        description: t("There was an error when trying to remove the member."),
        variant: "warning",
      });
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
      toast({
        title: t("Member's Role updated"),
        description: t("Member's Role has been successfully updated."),
        variant: "default",
      });
      return { workspace: data, ...rest };
    } catch (_err) {
      toast({
        title: t("Member's Role Could Not Be Updated"),
        description: t(
          "There was an error when trying to update the members persmissons.",
        ),
        variant: "warning",
      });
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
