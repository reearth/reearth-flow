import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useCallback } from "react";

import { useGraphQLContext } from "@flow/lib/gql";
import { Project } from "@flow/types";
import { isDefined } from "@flow/utils";

import {
  CreateProjectInput,
  DeleteProjectInput,
  UpdateProjectInput,
  ProjectFragment,
  RunProjectInput,
} from "../__gen__/graphql";

enum ProjectQueryKeys {
  GetWorkspaceProjects = "getWorkspaceProjects",
  GetProject = "getProject",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createNewProjectObject = useCallback(
    (project: ProjectFragment): Project => ({
      id: project.id,
      name: project.name,
      createdAt: project.createdAt,
      updatedAt: project.updatedAt,
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
    onSuccess: (project) =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, project?.workspaceId],
      }),
  });

  const useGetProjectsInfiniteQuery = (workspaceId?: string) =>
    useInfiniteQuery({
      queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      initialPageParam: null,
      queryFn: async ({ pageParam }) => {
        const data = await graphQLContext?.GetProjects({
          workspaceId: workspaceId ?? "",
          first: 5,
          after: pageParam,
        });
        if (!data) return;
        const {
          projects: {
            nodes,
            pageInfo: { endCursor, hasNextPage },
          },
        } = data;
        const projects: Project[] = nodes
          .filter(isDefined)
          .map((project) => createNewProjectObject(project));
        return { projects, endCursor, hasNextPage };
      },
      enabled: !!workspaceId,
      getNextPageParam: (lastPage) => {
        if (!lastPage) return undefined;
        const { endCursor, hasNextPage } = lastPage;
        return hasNextPage ? endCursor : undefined;
      },
    });

  const useGetProjectByIdQuery = (projectId?: string) =>
    useQuery({
      queryKey: [ProjectQueryKeys.GetProject, projectId],
      queryFn: () =>
        graphQLContext?.GetProjectById({ projectId: projectId ?? "" }),
      enabled: !!projectId,
      select: (data) =>
        data?.node?.__typename === "Project"
          ? createNewProjectObject(data.node)
          : undefined,
    });

  const updateProjectMutation = useMutation({
    mutationFn: async (input: UpdateProjectInput) => {
      const data = await graphQLContext?.UpdateProject({ input });

      if (data?.updateProject?.project) {
        return createNewProjectObject(data.updateProject.project);
      }
    },
    onSuccess: (project) =>
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
      const data = await graphQLContext?.DeleteProject({
        input: { projectId },
      });
      return {
        projectId: data?.deleteProject?.projectId,
        workspaceId: workspaceId,
      };
    },
    onSuccess: ({ workspaceId }) =>
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      }),
  });

  const runProjectMutation = useMutation({
    mutationFn: async ({
      projectId,
      workspaceId,
      workflows,
    }: RunProjectInput) => {
      const data = await graphQLContext?.RunProject({
        input: { projectId, workspaceId, workflows },
      });
      return {
        projectId: data?.runProject?.projectId,
        workspaceId: workspaceId,
        started: data?.runProject?.started,
      };
    },
    onSuccess: ({ workspaceId }) =>
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      }),
  });

  return {
    createProjectMutation,
    deleteProjectMutation,
    updateProjectMutation,
    runProjectMutation,
    useGetProjectsInfiniteQuery,
    useGetProjectByIdQuery,
  };
};
