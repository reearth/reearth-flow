import { useEffect, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";

import { useAuth } from "../auth";

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

  const [state, setState] = useState<{
    yDoc: Y.Doc;
    undoTrackerActionWrapper: (callback: () => void) => void;
  } | null>(null);
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
          setIsSynced(true); // Mark as synced
        });
      })();
    }

    // Initial state setup
    setState({
      yDoc,
      undoTrackerActionWrapper: (callback: () => void) =>
        yDoc.transact(callback, yDoc.clientID),
    });

    return () => {
      setIsSynced(false); // Mark as not synced
      yWebSocketProvider?.destroy(); // Cleanup on unmount
    };
  }, [projectId, workflowId, isProtected, getAccessToken]);

  const { yDoc, undoTrackerActionWrapper } = state || {};

  useEffect(() => {
    const yWorkflows = yDoc?.getArray<YWorkflow>("workflows");
    const currentUserClientId = yDoc?.clientID;
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
  }, [yDoc]);

  return {
    yDoc,
    isSynced,
    undoManager,
    undoTrackerActionWrapper,
  };
};
