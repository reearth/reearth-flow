import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

import { useGraphQLContext } from "@flow/lib/gql";
import { Project } from "@flow/types";
import { isDefined } from "@flow/utils";

import {
  CreateProjectInput,
  DeleteProjectInput,
  UpdateProjectInput,
  ProjectFragment,
} from "../__gen__/graphql";

import { ProjectQueryKeys } from "./useApi";

export const useQueries = () => {
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
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, project?.workspaceId],
      }),
  });

  const useGetProjectsQuery = (workspaceId?: string) =>
    useQuery({
      queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      queryFn: () => graphQLContext?.GetProjects({ workspaceId: workspaceId ?? "", first: 20 }),
      enabled: !!workspaceId,
      select: data => {
        if (!data) return {};
        const {
          projects: { nodes, ...rest },
        } = data;

        const projects: Project[] = nodes
          .filter(isDefined)
          .map(project => createNewProjectObject(project));
        return { projects, meta: rest };
      },
    });

  const useGetProjectByIdQuery = (projectId?: string) =>
    useQuery({
      queryKey: [ProjectQueryKeys.GetProject, projectId],
      queryFn: () => graphQLContext?.GetProjectById({ projectId: projectId ?? "" }),
      enabled: !!projectId,
      select: data =>
        data?.node?.__typename === "Project" ? createNewProjectObject(data.node) : undefined,
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
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, project?.workspaceId],
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
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      }),
  });

  return {
    createProjectMutation,
    useGetProjectsQuery,
    useGetProjectByIdQuery,
    deleteProjectMutation,
    updateProjectMutation,
  };
};
