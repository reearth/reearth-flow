import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { ProjectSnapshotMeta } from "@flow/types";
import { isDefined } from "@flow/utils";

import { toProjectDocument, toProjectSnapShotMeta } from "../convert";
import { useGraphQLContext } from "../provider";

export enum DocumentQueryKeys {
  GetLatestProjectSnapshot = "getLatestProjectSnapshot",
  GetProjectSnapshot = "getProjectSnapshot",
  GetProjectHistory = "getProjectHistory",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useLatestProjectSnapshotQuery = (projectId: string) =>
    useQuery({
      queryKey: [DocumentQueryKeys.GetLatestProjectSnapshot, projectId],
      queryFn: async () => {
        const data = await graphQLContext?.GetLatestProjectSnapshot({
          projectId,
        });
        if (!data?.latestProjectSnapshot) return;
        return toProjectDocument(data.latestProjectSnapshot);
      },
      enabled: !!projectId,
    });

  const useProjectHistoryQuery = (projectId: string) =>
    useQuery({
      queryKey: [DocumentQueryKeys.GetProjectHistory, projectId],
      queryFn: async () => {
        const data = await graphQLContext?.GetProjectHistory({
          projectId,
        });

        if (!data) return;
        const { projectHistory } = data;
        const history: ProjectSnapshotMeta[] = projectHistory
          .filter(isDefined)
          .map((projectSnapshot) => toProjectSnapShotMeta(projectSnapshot));

        return history;
      },
      enabled: !!projectId,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
    });

  const rollbackProjectMutation = useMutation({
    mutationFn: async ({
      projectId,
      version,
    }: {
      projectId: string;
      version: number;
    }) => {
      const data = await graphQLContext?.RollbackProject({
        projectId,
        version,
      });

      if (data?.rollbackProject) {
        return data?.rollbackProject;
      }
    },
    onSuccess: (projectDocument) => {
      if (projectDocument) {
        queryClient.invalidateQueries({
          queryKey: [
            DocumentQueryKeys.GetLatestProjectSnapshot,
            projectDocument.id,
          ],
        });
        queryClient.invalidateQueries({
          queryKey: [DocumentQueryKeys.GetProjectHistory, projectDocument.id],
        });
      }
    },
  });

  return {
    useLatestProjectSnapshotQuery,
    useProjectHistoryQuery,
    rollbackProjectMutation,
  };
};
