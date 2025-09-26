import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { Project } from "@flow/types";
import {
  OrderDirection,
  type PaginationOptions,
} from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import {
  CreateProjectInput,
  DeleteProjectInput,
  UpdateProjectInput,
  RunProjectInput,
} from "../__gen__/graphql";
import { toJob, toProject } from "../convert";

export enum ProjectQueryKeys {
  GetWorkspaceProjects = "getWorkspaceProjects",
  GetProject = "getProject",
}

export const PROJECT_FETCH_AMOUNT = 16;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createProjectMutation = useMutation({
    mutationFn: async (input: CreateProjectInput) => {
      const data = await graphQLContext?.CreateProject({ input });

      if (data?.createProject?.project) {
        return toProject(data.createProject.project);
      }
    },
    onSuccess: (project) =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, project?.workspaceId],
      }),
  });

  const useGetProjectsQuery = (
    workspaceId?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    return useQuery({
      queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetProjects({
          workspaceId: workspaceId ?? "",
          pagination: {
            page: paginationOptions?.page ?? 1,
            pageSize: PROJECT_FETCH_AMOUNT,
            orderDir: paginationOptions?.orderDir ?? OrderDirection.Desc,
            orderBy: paginationOptions?.orderBy ?? "updatedAt",
          },
        });
        if (!data) throw new Error("No data returned");
        const {
          projects: {
            nodes,
            pageInfo: { totalCount, currentPage, totalPages },
          },
        } = data;

        const projects: Project[] = nodes
          .filter(isDefined)
          .map((project) => toProject(project));

        return {
          projects,
          totalCount,
          currentPage,
          totalPages,
        };
      },
      enabled: !!workspaceId,
    });
  };

  const useGetProjectByIdQuery = (projectId?: string) =>
    useQuery({
      queryKey: [ProjectQueryKeys.GetProject, projectId],
      queryFn: () =>
        graphQLContext?.GetProjectById({ projectId: projectId ?? "" }),
      enabled: !!projectId,
      select: (data) =>
        data?.node?.__typename === "Project" ? toProject(data.node) : undefined,
    });

  const updateProjectMutation = useMutation({
    mutationFn: async (input: UpdateProjectInput) => {
      const data = await graphQLContext?.UpdateProject({ input });

      if (data?.updateProject?.project) {
        return toProject(data.updateProject.project);
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
    mutationFn: async ({ projectId, workspaceId, file }: RunProjectInput) => {
      const data = await graphQLContext?.RunProject({
        input: {
          projectId,
          workspaceId,
          file: file.get("file"),
        },
      });
      if (!data?.runProject?.job) return { workspaceId };
      return {
        job: toJob(data.runProject.job),
        workspaceId: workspaceId,
      };
    },
    onSuccess: ({ workspaceId }) =>
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      }),
  });

  const copyProjectMutation = useMutation({
    mutationFn: async ({
      projectId,
      source,
      workspaceId,
    }: {
      projectId: string;
      source: string;
      workspaceId: string;
    }) => {
      const data = await graphQLContext?.CopyProject({ projectId, source });
      return {
        success: data?.copyProject,
        workspaceId,
      };
    },
    onSuccess: ({ workspaceId }) =>
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      }),
  });

  const importProjectMutation = useMutation({
    mutationFn: async ({
      projectId,
      data: projectData,
      workspaceId,
    }: {
      projectId: string;
      data: Uint8Array;
      workspaceId: string;
    }) => {
      const data = await graphQLContext?.ImportProject({
        projectId,
        data: projectData,
      });
      return {
        success: data?.importProject,
        workspaceId,
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
    copyProjectMutation,
    importProjectMutation,
    useGetProjectsQuery,
    useGetProjectByIdQuery,
  };
};
