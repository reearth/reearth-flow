import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

import { useGraphQLContext } from "@flow/lib/gql";
import { Project } from "@flow/types";

import {
  CreateProjectInput,
  DeleteProjectInput,
  UpdateProjectInput,
  ProjectFragment,
} from "../__gen__/graphql";

import { ProjectQueryKeys } from "./useApi";

export const useFunction = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createNewProjectObject = useCallback(
    (project: ProjectFragment): Project => ({
      id: project.id,
      createdAt: project.createdAt,
      updatedAt: project.updatedAt,
      name: project.name,
      description: project.description,
      workspaceId: project.workspaceId,
    }),
    [],
  );

  const createProjectMutation = useMutation({
    mutationFn: async (input: CreateProjectInput) => {
      const data = await graphQLContext?.CreateProject({ input });

      if (data?.createProject?.project) {
        return createNewProjectObject(data.createProject.project);
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
          project ? [createNewProjectObject(project)] : [],
        );
        return { projects, meta: rest };
      },
    });

  const updateProjectMutation = useMutation({
    mutationFn: async (input: UpdateProjectInput) => {
      const data = await graphQLContext?.UpdateProject({ input });

      if (data?.updateProject?.project) {
        return createNewProjectObject(data.updateProject.project);
      }
    },
    onSuccess: project =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProjects, project?.workspaceId],
      }),
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
    updateProjectMutation,
  };
};
