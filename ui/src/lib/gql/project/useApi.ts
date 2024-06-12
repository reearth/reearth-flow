import { CreateProject, DeleteProject, GetProjects, UpdateProject } from "@flow/types";

import { CreateProjectInput, UpdateProjectInput } from "../__gen__/graphql";

import { useFunction } from "./useQueries";

export enum ProjectQueryKeys {
  GetProjects = "getProjects",
}

export const useProject = () => {
  const {
    createProjectMutation,
    useGetProjectsQuery,
    deleteProjectMutation,
    updateProjectMutation,
  } = useFunction();

  const createProject = async (input: CreateProjectInput): Promise<CreateProject> => {
    const { mutateAsync, ...rest } = createProjectMutation;
    try {
      const project = await mutateAsync(input);
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

  const updateProject = async (input: UpdateProjectInput): Promise<UpdateProject> => {
    const { mutateAsync, ...rest } = updateProjectMutation;
    try {
      const project = await mutateAsync(input);
      return { project, ...rest };
    } catch (err) {
      return { project: undefined, ...rest };
    }
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
    updateProject,
    deleteProject,
  };
};
