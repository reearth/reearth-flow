import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

import { useGraphQLContext } from "@flow/lib/gql";
import { Member, Workspace } from "@flow/types";

import { UpdateWorkspaceInput, WorkspaceFragment } from "../__gen__/graphql";

import { WorkspaceQueryKeys } from "./useApi";

export const useQueries = () => {
  // TODO: Move the react-query functions into it's own file.
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createNewWorkspaceObject = useCallback(
    (w: WorkspaceFragment): Workspace => ({
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
    }),
    [],
  );

  const createWorkspaceMutation = useMutation({
    mutationFn: async (name: string) => {
      const data = await graphQLContext?.CreateWorkspace({ input: { name } });
      return (
        data?.createWorkspace?.workspace &&
        createNewWorkspaceObject(data?.createWorkspace?.workspace)
      );
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: [WorkspaceQueryKeys.GetWorkspaces],
      });
    },
  });

  const useGetWorkspacesQuery = () =>
    useQuery({
      queryKey: [WorkspaceQueryKeys.GetWorkspaces],
      queryFn: () => graphQLContext?.GetWorkspaces(),
      select: data => data?.me?.workspaces.map(w => createNewWorkspaceObject(w)),
      staleTime: Infinity,
    });

  const useGetWorkspaceByIdQuery = (workspaceId: string) =>
    useQuery({
      queryKey: [WorkspaceQueryKeys.GetWorkspace, workspaceId],
      queryFn: () => graphQLContext?.GetWorkspaceById({ workspaceId }),
      select: data =>
        data?.node?.__typename === "Workspace" ? createNewWorkspaceObject(data.node) : undefined,
      staleTime: Infinity,
    });

  const updateWorkspaceMutation = useMutation({
    mutationFn: async (input: UpdateWorkspaceInput) => {
      const data = await graphQLContext?.UpdateWorkspace({ input });
      return (
        data?.updateWorkspace?.workspace &&
        createNewWorkspaceObject(data?.updateWorkspace?.workspace)
      );
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: [WorkspaceQueryKeys.GetWorkspaces],
      });
    },
  });

  const deleteWorkspaceMutation = useMutation({
    mutationFn: async (workspaceId: string) => {
      const data = await graphQLContext?.DeleteWorkspace({ input: { workspaceId } });
      return data?.deleteWorkspace?.workspaceId;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: [WorkspaceQueryKeys.GetWorkspaces],
      });
    },
  });

  return {
    createWorkspaceMutation,
    useGetWorkspacesQuery,
    useGetWorkspaceByIdQuery,
    deleteWorkspaceMutation,
    updateWorkspaceMutation,
  };
};
