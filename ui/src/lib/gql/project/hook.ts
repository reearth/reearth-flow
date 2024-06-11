import { CreateProject, DeleteProject, GetProjects } from "@flow/types";

import { CreateProjectInput } from "../__gen__/graphql";

import { useFunction } from "./function";

export enum ProjectQueryKeys {
  GetProjects = "getProjects",
}

export const useProject = () => {
  const { createProjectMutation, useGetProjectsQuery, deleteProjectMutation } = useFunction();

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
