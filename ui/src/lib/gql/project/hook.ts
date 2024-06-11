import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";
import { CreateProject, DeleteProject, GetProjects, Project } from "@flow/types";

import { CreateProjectInput, DeleteProjectInput } from "../__gen__/graphql";

export enum ProjectQueryKeys {
  GetProjects = "getProjects",
}

export const useProject = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createProjectMutation = useMutation({
    mutationFn: async (input: CreateProjectInput) => {
      const data = await graphQLContext?.CreateProject({ input });
      return { project: data?.createProject?.project };
    },
    onSuccess: project =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProjects, project.project?.workspaceId],
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

  const createProject = async (input: CreateProjectInput): Promise<CreateProject> => {
    const { mutateAsync, ...rest } = createProjectMutation;
    try {
      const { project: data } = await mutateAsync(input);
      const project = data && {
        id: data.id,
        createdAt: data.createdAt,
        updatedAt: data.updatedAt,
        name: data.name,
        description: data.description,
        workspaceId: data.workspaceId,
      };
      return { project, ...rest };
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

  const deleteProject = async (projectId: string, workspaceId: string): Promise<DeleteProject> => {
    const { mutateAsync, ...rest } = deleteProjectMutation;
    try {
      const data = await mutateAsync({ projectId, workspaceId });
      return { projectId: data.projectId, ...rest };
    } catch (err) {
      return { projectId: undefined, ...rest };
    }
  };

  return {
    useGetProjects,
    createProject,
    deleteProject,
  };
};
