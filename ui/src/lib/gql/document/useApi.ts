import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { RollbackProject } from "@flow/types";

import { useQueries } from "./useQueries";

export const useDocument = () => {
  const {
    useProjectSnapshotQuery,
    useLatestProjectSnapshotQuery,
    useProjectHistoryQuery,
    rollbackProjectMutation,
  } = useQueries();

  const { toast } = useToast();
  const t = useT();

  const useGetProjectSnapshot = (projectId: string, version: number) => {
    const { data, ...rest } = useProjectSnapshotQuery(projectId, version);
    return {
      projectDocument: data,
      ...rest,
    };
  };

  const useGetLatestProjectSnapshot = (projectId: string) => {
    const { data, ...rest } = useLatestProjectSnapshotQuery(projectId);
    return {
      projectDocument: data,
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

  return {
    useGetProjectSnapshot,
    useGetLatestProjectSnapshot,
    useGetProjectHistory,
    useRollbackProject,
  };
};
