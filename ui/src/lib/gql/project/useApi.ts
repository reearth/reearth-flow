import {
  CreateProject,
  DeleteProject,
  GetProject,
  GetWorkspaceProjects,
  GetWorkspaceProjectsInfinite,
  UpdateProject,
} from "@flow/types";

import { CreateProjectInput, UpdateProjectInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useProject = () => {
  const {
    createProjectMutation,
    useGetProjectsQuery,
    useGetProjectsInfiniteQuery,
    useGetProjectByIdQuery,
    deleteProjectMutation,
    updateProjectMutation,
  } = useQueries();

  const createProject = async (
    input: CreateProjectInput
  ): Promise<CreateProject> => {
    const { mutateAsync, ...rest } = createProjectMutation;
    try {
      const project = await mutateAsync(input);
      return { project, ...rest };
    } catch (_err) {
      return { project: undefined, ...rest };
    }
  };

  const useGetWorkspaceProjects = (
    workspaceId?: string
  ): GetWorkspaceProjects => {
    const { data, ...rest } = useGetProjectsQuery(workspaceId);
    return {
      projects: data?.projects,
      ...data?.meta,
      ...rest,
    };
  };

  const useGetWorkspaceProjectsInfinite = (
    workspaceId?: string
  ): GetWorkspaceProjectsInfinite => {
    const { data, ...rest } = useGetProjectsInfiniteQuery(workspaceId);
    return {
      pages: data?.pages,
      ...rest,
    };
  };

  const useGetProject = (projectId?: string): GetProject => {
    const { data, ...rest } = useGetProjectByIdQuery(projectId);
    return {
      project: data,
      ...rest,
    };
  };

  const updateProject = async (
    input: UpdateProjectInput
  ): Promise<UpdateProject> => {
    const { mutateAsync, ...rest } = updateProjectMutation;
    try {
      const project = await mutateAsync(input);
      return { project, ...rest };
    } catch (_err) {
      return { project: undefined, ...rest };
    }
  };

  const deleteProject = async (
    projectId: string,
    workspaceId: string
  ): Promise<DeleteProject> => {
    const { mutateAsync, ...rest } = deleteProjectMutation;
    try {
      const data = await mutateAsync({ projectId, workspaceId });
      return { projectId: data.projectId, ...rest };
    } catch (_err) {
      return { projectId: undefined, ...rest };
    }
  };

  return {
    useGetWorkspaceProjects,
    useGetWorkspaceProjectsInfinite,
    useGetProject,
    createProject,
    updateProject,
    deleteProject,
  };
};
