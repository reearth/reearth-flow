import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { NamedSnapshot, ProjectSnapshotMeta } from "@flow/types";
import { isDefined } from "@flow/utils";

import {
  toProjectDocument,
  toProjectSnapShot,
  toProjectSnapShotMeta,
} from "../convert";
import { useGraphQLContext } from "../provider";

export enum DocumentQueryKeys {
  GetLatestProjectSnapshot = "getLatestProjectSnapshot",
  GetProjectSnapshot = "getProjectSnapshot",
  GetProjectHistory = "getProjectHistory",
  GetProjectSnapshots = "getProjectSnapshots",
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

  const useProjectSnapshotQuery = (projectId: string, version: number) =>
    useQuery({
      queryKey: [DocumentQueryKeys.GetProjectSnapshot, projectId],
      queryFn: async () => {
        const data = await graphQLContext?.GetProjectSnapshot({
          projectId,
          version,
        });
        if (!data?.projectSnapshot) return;
        return toProjectSnapShot(data.projectSnapshot);
      },
      enabled: !!projectId && version != null,
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

  // Version history is snapshot-backed. The raw CRDT update log (projectHistory
  // above) is a durability concern and is deliberately not surfaced in the
  // panel: it has one entry per flush.
  const useProjectSnapshotsQuery = (projectId?: string) => {
    const { data, ...rest } = useQuery({
      queryKey: [DocumentQueryKeys.GetProjectSnapshots, projectId],
      queryFn: async (): Promise<NamedSnapshot[]> => {
        if (!projectId) return [];
        const data = await graphQLContext?.GetProjectSnapshots({ projectId });
        return data?.projectSnapshots ?? [];
      },
      enabled: !!projectId,
    });
    return { snapshots: data ?? [], ...rest };
  };

  const usePreviewSnapshot = useMutation({
    mutationFn: async ({
      projectId,
      version,
    }: {
      projectId: string;
      version: number;
    }) => {
      const data = await graphQLContext?.PreviewSnapshot({
        projectId,
        version,
      });

      if (data?.previewSnapshot) {
        return data?.previewSnapshot;
      }
    },
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

  const snapshotSaveMutation = useMutation({
    mutationFn: async ({ projectId }: { projectId: string }) => {
      const data = await graphQLContext?.SaveSnapshot({
        projectId,
      });

      if (data?.saveSnapshot) {
        return data.saveSnapshot;
      }
    },
  });

  return {
    useLatestProjectSnapshotQuery,
    useProjectSnapshotQuery,
    useProjectHistoryQuery,
    useProjectSnapshotsQuery,
    usePreviewSnapshot,
    rollbackProjectMutation,
    snapshotSaveMutation,
  };
};
