import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import type { Project } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import type { PaginationOptions } from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import type {
  CreateProjectInput,
  DeleteProjectInput,
  UpdateProjectInput,
  RunProjectInput,
  PreviewSchemaInput,
  SetProjectStarInput,
} from "../__gen__/graphql";
import { toJob, toProject } from "../convert";

export enum ProjectQueryKeys {
  GetWorkspaceProjects = "getWorkspaceProjects",
  GetProject = "getProject",
  CheckProjectAlias = "checkProjectAlias",
}

type WorkspaceProjectsPage = {
  projects: Project[];
  totalCount: number;
  currentPage: number | null;
  totalPages: number | null;
};

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
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    return useQuery({
      queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetProjects({
          workspaceId: workspaceId ?? "",
          keyword,
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

        const page: WorkspaceProjectsPage = {
          projects,
          totalCount,
          currentPage,
          totalPages,
        };
        return page;
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
    onSuccess: (project) => {
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, project?.workspaceId],
      });
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProject, project?.id],
      });
    },
  });

  // Rewrites the project wherever a cached projects page holds it, so a star
  // shows at once without refetching (and possibly reordering) the page.
  const patchCachedProject = (
    projectId: string,
    patch: (project: Project) => Project,
  ) =>
    queryClient.setQueriesData<WorkspaceProjectsPage>(
      { queryKey: [ProjectQueryKeys.GetWorkspaceProjects] },
      (page) =>
        page && {
          ...page,
          projects: page.projects.map((p) =>
            p.id === projectId ? patch(p) : p,
          ),
        },
    );

  const setProjectStarMutation = useMutation({
    mutationFn: async (input: SetProjectStarInput) => {
      const data = await graphQLContext?.SetProjectStar({ input });
      if (data?.setProjectStar?.project) {
        return toProject(data.setProjectStar.project);
      }
    },
    onMutate: async ({ projectId, starred }) => {
      await queryClient.cancelQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects],
      });
      const previous = queryClient.getQueriesData<WorkspaceProjectsPage>({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects],
      });
      patchCachedProject(projectId, (p) =>
        p.starred === starred
          ? p
          : {
              ...p,
              starred,
              starCount: Math.max(0, p.starCount + (starred ? 1 : -1)),
            },
      );
      return { previous };
    },
    onError: (_err, _input, context) => {
      context?.previous.forEach(([queryKey, page]) =>
        queryClient.setQueryData(queryKey, page),
      );
    },
    onSuccess: (project, { projectId }) => {
      if (project) {
        patchCachedProject(projectId, (p) => ({
          ...p,
          starred: project.starred,
          starCount: project.starCount,
        }));
      }
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProject, projectId],
      });
    },
  });

  // A malformed alias is a GraphQL error rather than available: false, so it
  // is not retried.
  const useCheckProjectAliasQuery = (
    workspaceId: string | undefined,
    alias: string | undefined,
    projectId?: string,
  ) =>
    useQuery({
      queryKey: [
        ProjectQueryKeys.CheckProjectAlias,
        workspaceId,
        alias,
        projectId,
      ],
      queryFn: async () => {
        const data = await graphQLContext?.CheckProjectAlias({
          workspaceId: workspaceId ?? "",
          alias: alias ?? "",
          projectId,
        });
        if (!data) throw new Error("No data returned");
        return data.checkProjectAlias;
      },
      enabled: !!workspaceId && !!alias,
      retry: false,
      staleTime: 0,
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
      file,
      previousJobId,
      startNodeId,
    }: RunProjectInput) => {
      const data = await graphQLContext?.RunProject({
        input: {
          projectId,
          workspaceId,
          file: file.get("file"),
          previousJobId,
          startNodeId,
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

  const previewSchemaMutation = useMutation({
    mutationFn: async ({
      projectId,
      workspaceId,
      file,
      parameters,
      sampleSize,
    }: Omit<PreviewSchemaInput, "file"> & { file: FormData }) => {
      const data = await graphQLContext?.PreviewSchema({
        input: {
          projectId,
          workspaceId,
          file: file.get("file"),
          parameters,
          sampleSize,
        },
      });
      if (!data?.previewSchema?.job) return { workspaceId };
      return {
        job: toJob(data.previewSchema.job),
        workspaceId,
      };
    },
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
      const dataArray = Array.from(projectData);
      const data = await graphQLContext?.ImportProject({
        projectId,
        data: dataArray,
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
    previewSchemaMutation,
    copyProjectMutation,
    importProjectMutation,
    setProjectStarMutation,
    useGetProjectsQuery,
    useGetProjectByIdQuery,
    useCheckProjectAliasQuery,
  };
};
