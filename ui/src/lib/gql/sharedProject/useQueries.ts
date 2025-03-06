import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { Project } from "@flow/types";

import { ShareProjectInput, UnshareProjectInput } from "../__gen__/graphql";
import { toProject } from "../convert";
import { ProjectQueryKeys } from "../project/useQueries";

export enum SharedProjectQueryKeys {
  GetSharedProject = "getSharedProject",
  GetSharedProjectInfo = "getSharedProjectInfo",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const shareProjectMutation = useMutation({
    mutationFn: async (input: ShareProjectInput & { workspaceId: string }) => {
      const data = await graphQLContext?.ShareProject({
        input: { projectId: input.projectId },
      });

      if (data?.shareProject) {
        return { ...data.shareProject, workspaceId: input.workspaceId };
      }
    },
    onSuccess: (data) =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, data?.workspaceId],
      }),
  });

  const unshareProjectMutation = useMutation({
    mutationFn: async (
      input: UnshareProjectInput & { workspaceId: string },
    ) => {
      const data = await graphQLContext?.UnshareProject({
        input: { projectId: input.projectId },
      });

      if (data?.unshareProject) {
        return { ...data.unshareProject, workspaceId: input.workspaceId };
      }
    },
    onSuccess: (data) =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, data?.workspaceId],
      }),
  });

  const useGetSharedProjectQuery = (token?: string) => {
    return useQuery({
      queryKey: [SharedProjectQueryKeys.GetSharedProject, token],
      queryFn: async () => {
        const data = await graphQLContext?.GetSharedProject({
          token: token ?? "",
        });
        if (!data) throw new Error("No data returned");
        const {
          sharedProject: { project: rawSharedProject },
        } = data;

        const sharedProject: Project = toProject(rawSharedProject);

        return {
          sharedProject,
        };
      },
      enabled: !!token,
    });
  };

  const useGetSharedProjectInfoQuery = (projectId?: string) => {
    return useQuery({
      queryKey: [SharedProjectQueryKeys.GetSharedProjectInfo, projectId],
      queryFn: async () => {
        const data = await graphQLContext?.GetSharedProjectInfo({
          projectId: projectId ?? "",
        });
        if (!data) throw new Error("No data returned");
        const { projectSharingInfo } = data;

        return {
          projectSharingInfo,
        };
      },
      enabled: !!projectId,
    });
  };

  return {
    shareProjectMutation,
    unshareProjectMutation,
    useGetSharedProjectQuery,
    useGetSharedProjectInfoQuery,
  };
};
