import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import {
  CreateProject,
  DeleteProject,
  GetProject,
  GetWorkspaceProjects,
  RunProject,
  UpdateProject,
} from "@flow/types";

import { CreateProjectInput, UpdateProjectInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useProject = () => {
  const { toast } = useToast();
  const t = useT();

  const {
    createProjectMutation,
    deleteProjectMutation,
    updateProjectMutation,
    runProjectMutation,
    useGetProjectsInfiniteQuery,
    useGetProjectByIdQuery,
  } = useQueries();

  const createProject = async (
    input: CreateProjectInput,
  ): Promise<CreateProject> => {
    const { mutateAsync, ...rest } = createProjectMutation;
    try {
      const project = await mutateAsync(input);
      toast({
        title: t("Project Created"),
        description: t("Project has been successfully created."),
      });
      return { project, ...rest };
    } catch (_err) {
      return { project: undefined, ...rest };
    }
  };

  const useGetWorkspaceProjectsInfinite = (
    workspaceId?: string,
  ): GetWorkspaceProjects => {
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
    input: UpdateProjectInput,
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
    workspaceId: string,
  ): Promise<DeleteProject> => {
    const { mutateAsync, ...rest } = deleteProjectMutation;
    try {
      const data = await mutateAsync({ projectId, workspaceId });
      toast({
        title: t("Successful Deletion"),
        description: t(
          "Project has been successfully deleted from your workspace.",
        ),
        variant: "destructive",
      });
      return { projectId: data.projectId, ...rest };
    } catch (_err) {
      return { projectId: undefined, ...rest };
    }
  };

  const runProject = async (
    projectId: string,
    workspaceId: string,
    workflow: string,
  ): Promise<RunProject> => {
    const { mutateAsync, ...rest } = runProjectMutation;

    try {
      const data = await mutateAsync({
        projectId,
        workspaceId,
        file: workflow,
      });
      toast({
        title: t("Successful Deletion"),
        description: t(
          "Project has been successfully deleted from your workspace.",
        ),
        variant: "destructive",
      });
      console.log("data", data);
      return { projectId: data.projectId, started: data.started, ...rest };
    } catch (_err) {
      console.log("Errror", _err);
      return { projectId: undefined, ...rest };
    }
  };

  return {
    useGetWorkspaceProjectsInfinite,
    useGetProject,
    createProject,
    updateProject,
    deleteProject,
    runProject,
  };
};
