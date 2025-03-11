import { useCallback, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import { Doc } from "yjs";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useAuth } from "@flow/lib/auth";
import { useDocument } from "@flow/lib/gql/document/useApi";

export default ({
  projectId,
  yDoc,
}: {
  projectId: string;
  yDoc: Doc | undefined;
}) => {
  const { getAccessToken } = useAuth();
  const { websocket } = config();
  const {
    useGetProjectHistory,
    useGetLatestProjectSnapshot,
    useRollbackProject,
  } = useDocument();

  const { history, isFetching } = useGetProjectHistory(projectId);

  const { projectDocument } = useGetLatestProjectSnapshot(projectId);

  const [selectedProjectSnapshotVersion, setSelectedProjectSnapshotVersion] =
    useState<number | null>(null);
  const [openVersionChangeDialog, setOpenVersionChangeDialog] =
    useState<boolean>(false);

  const handleRollbackProject = useCallback(async () => {
    if (selectedProjectSnapshotVersion === null) return;
    const rollbackData = await useRollbackProject(
      projectId,
      selectedProjectSnapshotVersion,
    );
    const updates = rollbackData.projectDocument?.updates;

    try {
      if (!updates || !updates.length) {
        return;
      }
      const convertedUpdates = new Uint8Array(updates);
      if (websocket && yDoc) {
        const token = await getAccessToken();
        const yWebSocketProvider = new WebsocketProvider(
          websocket,
          `${projectId}:${DEFAULT_ENTRY_GRAPH_ID}`,
          yDoc,
          { params: { token } },
        );
        await new Promise<void>((resolve) => {
          yWebSocketProvider.once("sync", () => {
            yDoc.transact(() => {
              Y.applyUpdate(yDoc, convertedUpdates);
            });
            resolve();
          });
        });
      }
    } catch (error) {
      console.error("Project RollBack:", error);
    }

    setOpenVersionChangeDialog(false);
  }, [
    selectedProjectSnapshotVersion,
    useRollbackProject,
    projectId,
    getAccessToken,
    websocket,
    yDoc,
  ]);

  const latestProjectSnapshotVersion = projectDocument;
  return {
    history,
    isFetching,
    latestProjectSnapshotVersion,
    selectedProjectSnapshotVersion,
    setSelectedProjectSnapshotVersion,
    openVersionChangeDialog,
    setOpenVersionChangeDialog,
    onRollbackProject: handleRollbackProject,
  };
};
