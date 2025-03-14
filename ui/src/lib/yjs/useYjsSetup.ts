import { useEffect, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { useAuth } from "../auth";

import { yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";

export default ({
  workflowId,
  projectId,
  isProtected,
}: {
  workflowId?: string;
  projectId?: string;
  isProtected?: boolean;
}) => {
  const { getAccessToken } = useAuth();

  const [undoManager, setUndoManager] = useState<Y.UndoManager | null>(null);

  const [yDocState, setYDocState] = useState<Y.Doc | null>(null);
  const [isSynced, setIsSynced] = useState(false);

  useEffect(() => {
    const yDoc = new Y.Doc();
    const { websocket } = config();
    let yWebSocketProvider: WebsocketProvider | null = null;

    if (workflowId && websocket && projectId) {
      (async () => {
        const params: Record<string, string> = {};
        if (isProtected) {
          const token = await getAccessToken();
          params.token = token;
        }

        yWebSocketProvider = new WebsocketProvider(
          websocket,
          `${projectId}:${workflowId}`,
          yDoc,
          {
            params,
          },
        );

        yWebSocketProvider.once("sync", () => {
          const metadata = yDoc.getMap("metadata");
          if (!metadata.get("initialized")) {
            // Within a transaction, set the flag and perform initialization.
            yDoc.transact(() => {
              const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
              // This check is only necessary to avoid duplicate workflows on older projects.
              if (yWorkflows.length > 0) return;
              // Only one client should set this flag.
              if (!metadata.get("initialized")) {
                const yWorkflow = yWorkflowConstructor(
                  DEFAULT_ENTRY_GRAPH_ID,
                  "Main Workflow",
                );
                yWorkflows.insert(0, [yWorkflow]);
                metadata.set("initialized", true);
              }
            });
          }
          setIsSynced(true); // Mark as synced
        });
      })();
    }

    setYDocState(yDoc);

    return () => {
      setIsSynced(false);
      yWebSocketProvider?.destroy();
    };
  }, [projectId, workflowId, isProtected, getAccessToken]);

  const currentUserClientId = yDocState?.clientID;

  const yWorkflows = yDocState?.getArray<YWorkflow>("workflows");

  const undoTrackerActionWrapper = (callback: () => void) =>
    yDocState?.transact(callback, yDocState.clientID);

  useEffect(() => {
    if (yWorkflows) {
      const manager = new Y.UndoManager(yWorkflows, {
        trackedOrigins: new Set([currentUserClientId]), // Only track local changes
        captureTimeout: 200, // default is 500. 200ms is a good balance between performance and user experience
      });
      setUndoManager(manager);

      return () => {
        manager.destroy(); // Clean up UndoManager on component unmount
      };
    }
  }, [yWorkflows, currentUserClientId]);

  return {
    yWorkflows,
    isSynced,
    undoManager,
    undoTrackerActionWrapper,
    yDocState,
  };
};
