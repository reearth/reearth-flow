import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import {
  CreateProject,
  DeleteProject,
  EngineReadyWorkflow,
  GetProject,
  Project,
  RunProject,
  UpdateProject,
} from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";
import { jsonToFormData } from "@flow/utils/jsonToFormData";

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
    useGetProjectsQuery,
    useGetProjectByIdQuery,
  } = useQueries();

  const createProject = async (
    input: CreateProjectInput,
  ): Promise<CreateProject> => {
    const { mutateAsync, ...rest } = createProjectMutation;
    try {
      const project: Project | undefined = await mutateAsync(input);
      toast({
        title: t("Project Created"),
        description: t("Project has been successfully created."),
      });
      return { project, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Could Not Be Created"),
        description: t("There was an error when creating the project."),
        variant: "destructive",
      });
      return { project: undefined, ...rest };
    }
  };

  const useGetWorkspaceProjects = (
    workspaceId?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetProjectsQuery(
      workspaceId,
      paginationOptions,
    );
    return {
      page: data,
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
      toast({
        title: t("Project Could Not Be Updated"),
        description: t("There was an error when updating the project."),
        variant: "destructive",
      });
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
      });
      return { projectId: data.projectId, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Could Not Be Deleted"),
        description: t("There was an error when deleting the project."),
        variant: "destructive",
      });
      return { projectId: undefined, ...rest };
    }
  };

  const runProject = async (
    projectId: string,
    workspaceId: string,
    engineReadyWorkflow: EngineReadyWorkflow,
  ): Promise<RunProject> => {
    const { mutateAsync, ...rest } = runProjectMutation;

    try {
      const formData = jsonToFormData(
        engineReadyWorkflow,
        engineReadyWorkflow.id,
      );
      const data = await mutateAsync({
        projectId,
        workspaceId,
        file: formData,
      });
      toast({
        title: t("Debug run started"),
        description: t(
          "Debug run has been successfully started for the project.",
        ),
      });
      return { job: data.job, ...rest };
    } catch (err) {
      console.error("error", err);
      toast({
        title: t("Debug Run Could Not Be Started"),
        description: t(
          "There was an error when attempting to run the current workflow.",
        ),
        variant: "destructive",
      });
      return { job: undefined, ...rest };
    }
  };

  return {
    useGetWorkspaceProjects,
    useGetProject,
    createProject,
    updateProject,
    deleteProject,
    runProject,
  };
};
