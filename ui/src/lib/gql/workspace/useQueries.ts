import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

import { useGraphQLContext } from "@flow/lib/gql";
import { Member, Workspace } from "@flow/types";
import { isDefined } from "@flow/utils";

import {
  AddMemberToWorkspaceInput,
  RemoveMemberFromWorkspaceInput,
  UpdateMemberOfWorkspaceInput,
  UpdateWorkspaceInput,
  WorkspaceFragment,
} from "../__gen__/graphql";

import { WorkspaceQueryKeys } from "./useApi";

export const useQueries = () => {
  // TODO: Move the react-query functions into it's own file.
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createNewWorkspaceObject = useCallback((w?: WorkspaceFragment): Workspace | undefined => {
    if (!w) return;
    return {
      id: w.id,
      name: w.name,
      personal: w.personal,
      members: w.members.map(
        (m): Member => ({
          userId: m.userId,
          role: m.role,
          user: m.user
            ? {
                id: m.user?.id,
                name: m.user?.name,
                email: m.user?.email,
              }
            : undefined,
        }),
      ),
    };
  }, []);

  const updateWorkspace = (workspace?: Workspace) => {
    if (!workspace) return;
    queryClient.setQueryData([WorkspaceQueryKeys.GetWorkspaces], (data: Workspace[]) => {
      data.splice(
        data.findIndex(w => w.id === workspace?.id),
        1,
        workspace,
      );
      return [...data];
    });
    queryClient.setQueryData([WorkspaceQueryKeys.GetWorkspace, workspace.id], () => workspace);
  };

  const createWorkspaceMutation = useMutation({
    mutationFn: async (name: string) => {
      const data = await graphQLContext?.CreateWorkspace({ input: { name } });
      return createNewWorkspaceObject(data?.createWorkspace?.workspace);
    },
    onSuccess: createdWorkspace => {
      queryClient.setQueryData([WorkspaceQueryKeys.GetWorkspaces], (data: Workspace[]) => [
        ...data,
        createdWorkspace,
      ]);
    },
  });

  const useGetWorkspacesQuery = () =>
    useQuery({
      queryKey: [WorkspaceQueryKeys.GetWorkspaces],
      queryFn: async () => {
        const data = await graphQLContext?.GetWorkspaces();

        return data?.me?.workspaces
          .filter(isDefined)
          .map(w => createNewWorkspaceObject(w) as Workspace);
      },
      staleTime: Infinity,
    });

  const useGetWorkspaceByIdQuery = (workspaceId?: string) =>
    useQuery({
      queryKey: [WorkspaceQueryKeys.GetWorkspace, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetWorkspaceById({ workspaceId: workspaceId ?? "" });
        return data?.node?.__typename === "Workspace"
          ? createNewWorkspaceObject(data.node)
          : undefined;
      },
      enabled: !!workspaceId,
      staleTime: Infinity,
    });

  const updateWorkspaceMutation = useMutation({
    mutationFn: async (input: UpdateWorkspaceInput) => {
      const data = await graphQLContext?.UpdateWorkspace({ input });
      return createNewWorkspaceObject(data?.updateWorkspace?.workspace);
    },
    onSuccess: updateWorkspace,
  });

  const deleteWorkspaceMutation = useMutation({
    mutationFn: async (workspaceId: string) => {
      const data = await graphQLContext?.DeleteWorkspace({ input: { workspaceId } });
      return data?.deleteWorkspace?.workspaceId;
    },
    onSuccess: deletedWorkspaceId => {
      queryClient.setQueryData([WorkspaceQueryKeys.GetWorkspaces], (data: Workspace[]) => {
        data.splice(
          data.findIndex(w => w.id === deletedWorkspaceId),
          1,
        );
        return [...data];
      });
    },
  });

  // Members in the Workspace
  const addMemberToWorkspaceMutation = useMutation({
    mutationFn: async (input: AddMemberToWorkspaceInput) => {
      const data = await graphQLContext?.AddMemberToWorkspace({
        input,
      });
      return createNewWorkspaceObject(data?.addMemberToWorkspace?.workspace);
    },
    onSuccess: updateWorkspace,
  });

  const removeMemberFromWorkspaceMutation = useMutation({
    mutationFn: async (input: RemoveMemberFromWorkspaceInput) => {
      const data = await graphQLContext?.RemoveMemberFromWorkspace({
        input,
      });
      return createNewWorkspaceObject(data?.removeMemberFromWorkspace?.workspace);
    },
    onSuccess: updateWorkspace,
  });

  const updateMemberOfWorkspaceMutation = useMutation({
    mutationFn: async (input: UpdateMemberOfWorkspaceInput) => {
      const data = await graphQLContext?.UpdateMemberOfWorkspace({
        input,
      });
      return createNewWorkspaceObject(data?.updateMemberOfWorkspace?.workspace);
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
