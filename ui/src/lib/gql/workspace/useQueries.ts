import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

import { useGraphQLContext } from "@flow/lib/gql";
import { Workspace } from "@flow/types";

import { GetWorkspaceFragment } from "../__gen__/graphql";

import { WorkspaceQueryKeys } from "./useApi";

export const useQueries = () => {
  // TODO: Move the react-query functions into it's own file.
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createNewWorkspaceObject = useCallback(
    (w: GetWorkspaceFragment): Workspace => ({
      id: w.id,
      name: w.name,
      personal: w.personal,
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
    onSuccess: createdWorkspace => {
      queryClient.setQueryData([WorkspaceQueryKeys.GetWorkspace], (data: Workspace[]) => [
        ...data,
        createdWorkspace,
      ]);
    },
  });

  const getWorkspacesQuery = useQuery({
    queryKey: [WorkspaceQueryKeys.GetWorkspace],
    queryFn: () => graphQLContext?.GetWorkspaces(),
    select: data => data?.me?.workspaces.map(w => createNewWorkspaceObject(w)),
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

  return {
    createWorkspaceMutation,
    getWorkspacesQuery,
    deleteWorkspaceMutation,
  };
};
