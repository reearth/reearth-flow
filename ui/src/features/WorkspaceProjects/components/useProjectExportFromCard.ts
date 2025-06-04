import { useCallback, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useProjectExport } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { yWorkflowConstructor } from "@flow/lib/yjs/conversions";
import { YWorkflow } from "@flow/lib/yjs/types";
import { Project } from "@flow/types";

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
            const metadata = yDoc.getMap("metadata");
            if (!metadata.get("initialized")) {
              yDoc.transact(() => {
                const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
                if (yWorkflows.get(DEFAULT_ENTRY_GRAPH_ID)) return;
                if (!metadata.get("initialized")) {
                  const yWorkflow = yWorkflowConstructor(
                    DEFAULT_ENTRY_GRAPH_ID,
                    "Main Workflow",
                  );
                  yWorkflows.set(DEFAULT_ENTRY_GRAPH_ID, yWorkflow);
                  metadata.set("initialized", true);
                }
              });
            }
            await handleProjectExport({
              yDoc,
              project,
            });
            setIsExporting(false);

            yWebSocketProvider?.destroy();
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
