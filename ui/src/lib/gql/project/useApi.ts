import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateProject,
  DeleteProject,
  EngineReadyWorkflow,
  GetProject,
  PreviewSchema,
  Project,
  RunProject,
  UpdateProject,
} from "@flow/types";
import { PROJECT_MAX_TOPIC_LENGTH, PROJECT_MAX_TOPICS } from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";
import { jsonToFormData } from "@flow/utils/jsonToFormData";

import type {
  CreateProjectInput,
  UpdateProjectInput,
} from "../__gen__/graphql";

import { toProjectErrorKind } from "./errors";
import type { ProjectErrorKind } from "./errors";
import { useQueries } from "./useQueries";

export const useProject = () => {
  const { toast } = useToast();
  const t = useT();

  const {
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
  } = useQueries();

  const projectErrorMessage = (kind: ProjectErrorKind): string => {
    switch (kind) {
      case "privateNotAllowed":
        return t(
          "This workspace's plan does not allow private projects. Private projects need a paid team plan.",
        );
      case "aliasTaken":
        return t("This alias is already used by another project.");
      case "aliasReserved":
        return t('Aliases starting with "p-" are reserved.');
      case "aliasInvalid":
        return t(
          "Use 5 to 32 letters, numbers, hyphens (-) or underscores (_).",
        );
      case "tooManyTopics":
        return t("A project can have at most {{max}} topics.", {
          max: PROJECT_MAX_TOPICS,
        });
      case "topicTooLong":
        return t("Each topic can be at most {{max}} characters.", {
          max: PROJECT_MAX_TOPIC_LENGTH,
        });
    }
  };

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
    } catch (err) {
      const kind = toProjectErrorKind(err);
      const errorMessage = kind ? projectErrorMessage(kind) : undefined;
      toast({
        title: t("Project Could Not Be Created"),
        description:
          errorMessage ?? t("There was an error when creating the project."),
        variant: "destructive",
      });
      return { project: undefined, errorMessage, ...rest };
    }
  };

  const useGetWorkspaceProjects = (
    workspaceId?: string,
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetProjectsQuery(
      workspaceId,
      keyword,
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
      toast({
        title: t("Project Updated"),
        description: t("Project has been successfully updated."),
      });
      return { project, ...rest };
    } catch (err) {
      const kind = toProjectErrorKind(err);
      const errorMessage = kind ? projectErrorMessage(kind) : undefined;
      toast({
        title: t("Project Could Not Be Updated"),
        description:
          errorMessage ?? t("There was an error when updating the project."),
        variant: "destructive",
      });
      return { project: undefined, errorMessage, ...rest };
    }
  };

  // Sets rather than toggles, so a double click cannot undo itself.
  const setProjectStar = async (projectId: string, starred: boolean) => {
    try {
      await setProjectStarMutation.mutateAsync({ projectId, starred });
    } catch (_err) {
      toast({
        title: t("Star Could Not Be Updated"),
        description: t("There was an error when updating the project's star."),
        variant: "destructive",
      });
    }
  };

  const useCheckProjectAlias = (
    workspaceId: string | undefined,
    alias: string | undefined,
    projectId?: string,
  ) => {
    const { data, error, ...rest } = useCheckProjectAliasQuery(
      workspaceId,
      alias,
      projectId,
    );
    const errorKind = error ? toProjectErrorKind(error) : undefined;
    return {
      availability: data,
      errorMessage: error
        ? errorKind
          ? projectErrorMessage(errorKind)
          : t("Could not check whether this alias is available.")
        : undefined,
      ...rest,
    };
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
    previousJobId?: string,
    startNodeId?: string,
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
        previousJobId,
        startNodeId,
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

  const previewSchema = async (
    projectId: string,
    workspaceId: string,
    engineReadyWorkflow: EngineReadyWorkflow,
    sampleSize?: number,
  ): Promise<PreviewSchema> => {
    const { mutateAsync, ...rest } = previewSchemaMutation;

    try {
      const formData = jsonToFormData(
        engineReadyWorkflow,
        engineReadyWorkflow.id,
      );
      const data = await mutateAsync({
        projectId,
        workspaceId,
        file: formData,
        sampleSize,
      });
      // Intentionally quiet: this fires automatically on reader param save,
      // so a success toast on every save would be noisy.
      return { job: data.job, ...rest };
    } catch (err) {
      console.error("error", err);
      toast({
        title: t("Schema Preview Failed"),
        description: t(
          "There was an error when probing the reader's attribute schema.",
        ),
        variant: "destructive",
      });
      return { job: undefined, ...rest };
    }
  };

  const copyProject = async (
    projectId: string,
    source: string,
    workspaceId: string,
  ): Promise<{ success?: boolean }> => {
    const { mutateAsync, ...rest } = copyProjectMutation;
    try {
      const data = await mutateAsync({ projectId, source, workspaceId });
      toast({
        title: t("Project Copied"),
        description: t("Project has been successfully copied."),
      });
      return { success: data.success, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Could Not Be Copied"),
        description: t("There was an error when copying the project."),
        variant: "destructive",
      });
      return { success: false, ...rest };
    }
  };

  const importProject = async (
    projectId: string,
    data: Uint8Array,
    workspaceId: string,
  ): Promise<{ success?: boolean }> => {
    const { mutateAsync, ...rest } = importProjectMutation;
    try {
      const result = await mutateAsync({ projectId, data, workspaceId });
      toast({
        title: t("Project Imported"),
        description: t("Project has been successfully imported."),
      });
      return { success: result.success, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Could Not Be Imported"),
        description: t("There was an error when importing the project."),
        variant: "destructive",
      });
      return { success: false, ...rest };
    }
  };

  return {
    useGetWorkspaceProjects,
    useGetProject,
    useCheckProjectAlias,
    createProject,
    updateProject,
    deleteProject,
    runProject,
    previewSchema,
    copyProject,
    importProject,
    setProjectStar,
  };
};
