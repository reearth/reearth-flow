import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { RollbackProject, SaveSnapshot } from "@flow/types";

import { useQueries } from "./useQueries";

export const useDocument = () => {
  const {
    useLatestProjectSnapshotQuery,
    useProjectSnapshotQuery,
    useProjectHistoryQuery,
    rollbackProjectMutation,
    snapshotSaveMutation,
    usePreviewSnapshot,
  } = useQueries();

  const { toast } = useToast();
  const t = useT();

  const useGetLatestProjectSnapshot = (projectId: string) => {
    const { data, ...rest } = useLatestProjectSnapshotQuery(projectId);
    return {
      projectDocument: data,
      ...rest,
    };
  };

  const useGetProjectSnapshot = (projectId: string, version: number) => {
    const { data, ...rest } = useProjectSnapshotQuery(projectId, version);
    return {
      projectSnapshot: data,
      ...rest,
    };
  };

  const useGetProjectHistory = (projectId: string) => {
    const { data, ...rest } = useProjectHistoryQuery(projectId);
    return {
      history: data,
      ...rest,
    };
  };

  const useGetPreviewProjectSnapshot = async (
    projectId: string,
    version: number,
  ) => {
    const { mutateAsync, ...rest } = usePreviewSnapshot;
    try {
      const previewSnapshot = await mutateAsync({
        projectId,
        version,
      });

      return { previewSnapshot, ...rest };
    } catch (_err) {
      return { previewSnapshot: undefined, ...rest };
    }
  };

  const useRollbackProject = async (
    projectId: string,
    version: number,
  ): Promise<RollbackProject> => {
    const { mutateAsync, ...rest } = rollbackProjectMutation;
    try {
      const projectDocument = await mutateAsync({
        projectId,
        version,
      });

      toast({
        title: t("Project Rolled Back"),
        description: t(
          "Project has been successfully rolled back to version {{version}}.",
          { version },
        ),
      });

      return { projectDocument, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Rollback Failed"),
        description: t("There was an error rolling back the project."),
        variant: "warning",
      });

      return { projectDocument: undefined, ...rest };
    }
  };

  const useSaveSnapshot = async (projectId: string): Promise<SaveSnapshot> => {
    const { mutateAsync, ...rest } = snapshotSaveMutation;
    try {
      const saveSnapshot = await mutateAsync({
        projectId,
      });
      toast({
        title: t("Project Saved Successfully"),
        description: t("Project has been successfully saved."),
      });

      if (!saveSnapshot) return { saveSnapshot: false, ...rest };

      return { saveSnapshot, ...rest };
    } catch (_err) {
      toast({
        title: t("Project failed to save"),
        description: t("There was an error saving the project."),
        variant: "warning",
      });

      return { saveSnapshot: false, ...rest };
    }
  };

  return {
    useGetLatestProjectSnapshot,
    useGetProjectSnapshot,
    useGetProjectHistory,
    useGetPreviewProjectSnapshot,
    useRollbackProject,
    useSaveSnapshot,
  };
};
