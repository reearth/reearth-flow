import { useCallback, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useProjectExport } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { initializeProjectDoc } from "@flow/lib/yjs/initializeProjectDoc";
import type { Project } from "@flow/types";

export default (project: Project) => {
  const { getAccessToken } = useAuth();
  const { handleProjectExport } = useProjectExport();
  const [isExporting, setIsExporting] = useState<boolean>(false);

  const handleProjectExportFromCard = useCallback(async () => {
    try {
      setIsExporting(true);
      const yDoc = new Y.Doc();
      const { websocket } = config();
      let yWebSocketProvider: WebsocketProvider | null = null;
      if (websocket && project.id) {
        (async () => {
          const token = await getAccessToken();
          yWebSocketProvider = new WebsocketProvider(
            websocket,
            `${project.id}:${DEFAULT_ENTRY_GRAPH_ID}`,
            yDoc,
            {
              params: {
                token,
              },
            },
          );

          yWebSocketProvider.once("sync", async () => {
            // Exporting reads the project. Disconnect before giving a project
            // that was never opened its Main Workflow, so that only the
            // exported copy gets it and nothing is sent back to the server.
            yWebSocketProvider?.destroy();
            initializeProjectDoc(yDoc);
            await handleProjectExport({
              yDoc,
              project,
            });
            setIsExporting(false);
          });
        })();
      }
    } catch (error) {
      console.error("Project export failed:", error);
      setIsExporting(false);
    }
  }, [handleProjectExport, getAccessToken, project]);

  return {
    isExporting,
    handleProjectExportFromCard,
  };
};
