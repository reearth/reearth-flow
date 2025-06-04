import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import type { Workspace } from "@flow/types";
import { isDefined } from "@flow/utils";

import {
  AddMemberToWorkspaceInput,
  RemoveMemberFromWorkspaceInput,
  UpdateMemberOfWorkspaceInput,
  UpdateWorkspaceInput,
} from "../__gen__/graphql";
import { toWorkspace } from "../convert";

import { WorkspaceQueryKeys } from "./useApi";

export const useQueries = () => {
  // TODO: Move the react-query functions into it's own file.
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const updateWorkspace = (workspace?: Workspace) => {
    if (!workspace) return;
    queryClient.setQueryData(
      [WorkspaceQueryKeys.GetWorkspaces],
      (data: Workspace[]) => {
        data.splice(
          data.findIndex((w) => w.id === workspace?.id),
          1,
          workspace,
        );
        return [...data];
      },
    );
    queryClient.setQueryData(
      [WorkspaceQueryKeys.GetWorkspace, workspace.id],
      () => workspace,
    );
  };

  const createWorkspaceMutation = useMutation({
    mutationFn: async (name: string) => {
      const data = await graphQLContext?.CreateWorkspace({ input: { name } });
      if (!data?.createWorkspace?.workspace) return;
      return toWorkspace(data.createWorkspace.workspace);
    },
    onSuccess: (createdWorkspace) => {
      queryClient.setQueryData(
        [WorkspaceQueryKeys.GetWorkspaces],
        (data: Workspace[]) => [...data, createdWorkspace],
      );
    },
  });

  const useGetWorkspacesQuery = () =>
    useQuery({
      queryKey: [WorkspaceQueryKeys.GetWorkspaces],
      queryFn: async () => {
        const data = await graphQLContext?.GetWorkspaces();
        if (!data?.me?.workspaces) return;
        const workspaces: Workspace[] = data.me.workspaces
          .filter(isDefined)
          .map((workspace) => toWorkspace(workspace));
        return workspaces;
      },
      staleTime: Infinity,
    });

  const useGetWorkspaceByIdQuery = (workspaceId?: string) =>
    useQuery({
      queryKey: [WorkspaceQueryKeys.GetWorkspace, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetWorkspaceById({
          workspaceId: workspaceId ?? "",
        });
        if (!data?.node) return;
        return data.node?.__typename === "Workspace"
          ? toWorkspace(data.node)
          : undefined;
      },
      enabled: !!workspaceId,
      staleTime: Infinity,
    });

  const updateWorkspaceMutation = useMutation({
    mutationFn: async (input: UpdateWorkspaceInput) => {
      const data = await graphQLContext?.UpdateWorkspace({ input });
      if (!data?.updateWorkspace) return;

      return toWorkspace(data.updateWorkspace.workspace);
    },
    onSuccess: updateWorkspace,
  });

  const deleteWorkspaceMutation = useMutation({
    mutationFn: async (workspaceId: string) => {
      const data = await graphQLContext?.DeleteWorkspace({
        input: { workspaceId },
      });
      return data?.deleteWorkspace?.workspaceId;
    },
    onSuccess: (deletedWorkspaceId) => {
      queryClient.setQueryData(
        [WorkspaceQueryKeys.GetWorkspaces],
        (data: Workspace[]) => {
          data.splice(
            data.findIndex((w) => w.id === deletedWorkspaceId),
            1,
          );
          return [...data];
        },
      );
    },
  });

  // Members in the Workspace
  const addMemberToWorkspaceMutation = useMutation({
    mutationFn: async (input: AddMemberToWorkspaceInput) => {
      const data = await graphQLContext?.AddMemberToWorkspace({
        input,
      });
      if (!data?.addMemberToWorkspace) return;
      return toWorkspace(data.addMemberToWorkspace.workspace);
    },
    onSuccess: updateWorkspace,
  });

  const removeMemberFromWorkspaceMutation = useMutation({
    mutationFn: async (input: RemoveMemberFromWorkspaceInput) => {
      const data = await graphQLContext?.RemoveMemberFromWorkspace({
        input,
      });
      if (!data?.removeMemberFromWorkspace) return;

      return toWorkspace(data.removeMemberFromWorkspace.workspace);
    },
    onSuccess: updateWorkspace,
  });

  const updateMemberOfWorkspaceMutation = useMutation({
    mutationFn: async (input: UpdateMemberOfWorkspaceInput) => {
      const data = await graphQLContext?.UpdateMemberOfWorkspace({
        input,
      });
      if (!data?.updateMemberOfWorkspace) return;

      return toWorkspace(data.updateMemberOfWorkspace.workspace);
    },
    onSuccess: updateWorkspace,
  });

  return {
    createWorkspaceMutation,
    useGetWorkspacesQuery,
    useGetWorkspaceByIdQuery,
    deleteWorkspaceMutation,
    updateWorkspaceMutation,
    addMemberToWorkspaceMutation,
    removeMemberFromWorkspaceMutation,
    updateMemberOfWorkspaceMutation,
  };
};
