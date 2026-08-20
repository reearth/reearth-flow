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
  GetProjectNamedSnapshots = "getProjectNamedSnapshots",
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
      queryKey: [DocumentQueryKeys.GetProjectNamedSnapshots, projectId],
      queryFn: async (): Promise<NamedSnapshot[]> => {
        if (!projectId) return [];
        const data = await graphQLContext?.GetProjectNamedSnapshots({
          projectId,
        });
        return data?.projectNamedSnapshots ?? [];
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

  // Fetched on demand rather than as a hook: a snapshot's state is only wanted
  // once a row is clicked, and it is the full document, not list metadata.
  const fetchProjectNamedSnapshot = async (
    projectId: string,
    snapshotNumber: number,
  ): Promise<Uint8Array | undefined> => {
    const data = await graphQLContext?.GetProjectNamedSnapshot({
      projectId,
      snapshotNumber,
    });
    const updates = data?.projectSnapshot?.updates;
    if (!updates?.length) return undefined;
    return new Uint8Array(updates);
  };

  // Used before a restore so the pre-restore state is recoverable. Auto-versioning
  // only runs every 15 minutes, so without this the state being replaced might
  // have no snapshot of its own to come back to.
  const saveNamedSnapshotMutation = useMutation({
    mutationFn: async ({
      projectId,
      label,
    }: {
      projectId: string;
      label: string;
    }) => {
      const data = await graphQLContext?.SaveNamedSnapshot({
        projectId,
        label,
      });
      return data?.saveNamedSnapshot;
    },
    onSuccess: (_data, { projectId }) => {
      // The new snapshot must appear in the panel that triggered it.
      queryClient.invalidateQueries({
        queryKey: [DocumentQueryKeys.GetProjectNamedSnapshots, projectId],
      });
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
    fetchProjectNamedSnapshot,
    saveNamedSnapshotMutation,
  };
};
