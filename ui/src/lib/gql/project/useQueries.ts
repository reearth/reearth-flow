import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { Project } from "@flow/types";
import { isDefined } from "@flow/utils";

import {
  CreateProjectInput,
  DeleteProjectInput,
  UpdateProjectInput,
  RunProjectInput,
} from "../__gen__/graphql";
import { toProject } from "../convert";

enum ProjectQueryKeys {
  GetWorkspaceProjects = "getWorkspaceProjects",
  GetProject = "getProject",
}

const PROJECT_FETCH_AMOUNT = 5;

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

  const useGetProjectsInfiniteQuery = (workspaceId?: string) =>
    useInfiniteQuery({
      queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      initialPageParam: null,
      queryFn: async ({ pageParam }) => {
        const data = await graphQLContext?.GetProjects({
          workspaceId: workspaceId ?? "",
          first: PROJECT_FETCH_AMOUNT,
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
          .map((project) => toProject(project));
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
