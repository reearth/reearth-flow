import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { CreateProject, GetProjects, Project } from "@flow/types";

import { CreateProjectInput } from "../__gen__/graphql";

export enum ProjectQueryKeys {
  GetProjects = "getProjects",
}

export const useProject = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useCreateProjectMutation = (workspaceId: string) =>
    useMutation({
      mutationFn: async (input: CreateProjectInput) => {
        const data = await graphQLContext?.CreateProject({ input });
        return { project: data?.createProject?.project };
      },
      onSuccess: () =>
        queryClient.invalidateQueries({
          queryKey: [ProjectQueryKeys.GetProjects, workspaceId],
        }),
    });

  const useGetProjectsQuery = (workspaceId: string) =>
    useQuery({
      queryKey: [ProjectQueryKeys.GetProjects, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetProjects({ workspaceId, first: 5 });
        if (!data) return {};
        const {
          projects: { nodes, ...rest },
        } = data;
        return { projects: nodes as Project[], meta: rest };
      },
    });

  const useCreateProject = async (input: CreateProjectInput): Promise<CreateProject> => {
    const { mutateAsync, ...rest } = useCreateProjectMutation(input.workspaceId);
    try {
      const data = await mutateAsync(input);
      return { project: data.project, ...rest };
    } catch (err) {
      return { project: undefined, ...rest };
    }
  };

  const useGetProjects = (workspaceId: string): GetProjects => {
    const { data, ...rest } = useGetProjectsQuery(workspaceId);
    return {
      projects: data?.projects,
      ...data?.meta,
      ...rest,
    };
  };

  return {
    useGetProjects,
    useCreateProject,
  };
};
