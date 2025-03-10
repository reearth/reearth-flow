import { useEffect, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { useAuth } from "../auth";

import { yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";

const INIT_FLAG_KEY = "workflow_initialized";

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

  const [state, setState] = useState<{
    yDoc: Y.Doc;
    yWorkflows: Y.Array<YWorkflow>;
    undoTrackerActionWrapper: (callback: () => void) => void;
  } | null>(null);
  const [isSynced, setIsSynced] = useState(false);

  useEffect(() => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");

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
          const initFlag = yDoc.getMap(INIT_FLAG_KEY);

          yDoc.transact(() => {
            // console.log(
            //   "initFlag.get('initialized')",
            //   initFlag.get("initialized"),
            // );
            if (!initFlag.get("initialized") && yWorkflows.length === 0) {
              const yWorkflow = yWorkflowConstructor(
                DEFAULT_ENTRY_GRAPH_ID,
                "Main Workflow",
              );
              yWorkflows.insert(0, [yWorkflow]);
              initFlag.set("initialized", true);
            }
          });
          setIsSynced(true);
        });
      })();
    }

    // Initial state setup
    setState({
      yDoc,
      yWorkflows,
      undoTrackerActionWrapper: (callback: () => void) =>
        yDoc.transact(callback, yDoc.clientID),
    });

    return () => {
      setIsSynced(false); // Mark as not synced
      yWebSocketProvider?.destroy(); // Cleanup on unmount
    };
  }, [projectId, workflowId, isProtected, getAccessToken]);

  const { yDoc, yWorkflows, undoTrackerActionWrapper } = state || {};

  const currentUserClientId = yDoc?.clientID;

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
    state,
    isSynced,
    undoManager,
    undoTrackerActionWrapper,
  };
};
