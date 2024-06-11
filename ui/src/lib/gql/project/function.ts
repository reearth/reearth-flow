import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { Project } from "@flow/types";

import { CreateProjectInput, DeleteProjectInput } from "../__gen__/graphql";

import { ProjectQueryKeys } from "./hook";

export const useFunction = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createProjectMutation = useMutation({
    mutationFn: async (input: CreateProjectInput) => {
      const data = await graphQLContext?.CreateProject({ input });

      if (data?.createProject?.project) {
        const project = data.createProject.project;
        return {
          id: project.id,
          createdAt: project.createdAt,
          updatedAt: project.updatedAt,
          name: project.name,
          description: project.description,
          workspaceId: project.workspaceId,
        };
      }
    },
    onSuccess: project =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProjects, project?.workspaceId],
      }),
  });

  const useGetProjectsQuery = (workspaceId: string) =>
    useQuery({
      queryKey: [ProjectQueryKeys.GetProjects, workspaceId],
      queryFn: () => graphQLContext?.GetProjects({ workspaceId, first: 20 }),
      select: data => {
        if (!data) return {};
        const {
          projects: { nodes, ...rest },
        } = data;

        const projects: Project[] = nodes.flatMap(project =>
          project
            ? [
                {
                  id: project.id,
                  createdAt: project.createdAt,
                  updatedAt: project.updatedAt,
                  name: project.name,
                  description: project.description,
                  workspaceId: project.workspaceId,
                },
              ]
            : [],
        );
        return { projects, meta: rest };
      },
    });

  const deleteProjectMutation = useMutation({
    mutationFn: async ({
      projectId,
      workspaceId,
    }: DeleteProjectInput & { workspaceId: string }) => {
      const data = await graphQLContext?.DeleteProject({ input: { projectId } });
      return { projectId: data?.deleteProject?.projectId, workspaceId: workspaceId };
    },
    onSuccess: ({ workspaceId }) =>
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProjects, workspaceId],
      }),
  });

  return {
    createProjectMutation,
    useGetProjectsQuery,
    deleteProjectMutation,
  };
};
