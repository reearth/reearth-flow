import { useEffect, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";

export default ({
  workflowId,
  projectId,
  workspaceId,
  accessToken,
}: {
  workflowId?: string;
  projectId?: string;
  workspaceId?: string;
  accessToken?: string;
}) => {
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

    if (workflowId && websocket && projectId && workspaceId) {
      let params: Record<string, string> | undefined;
      if (accessToken) {
        params = {
          token: accessToken,
        };
      }
      const docPath = `workspaces/${workspaceId}/projects/${projectId}`;
      yWebSocketProvider = new WebsocketProvider(
        websocket,
        docPath,
        yDoc,
        {
          params,
        },
      );

      yWebSocketProvider.once("sync", () => {
        if (yWorkflows.length === 0) {
          yDoc.transact(() => {
            const yWorkflow = yWorkflowConstructor(
              DEFAULT_ENTRY_GRAPH_ID,
              "Main Workflow",
            );
            yWorkflows.insert(0, [yWorkflow]);
          });
        }

        setIsSynced(true); // Mark as synced
      });
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
  }, [projectId, workflowId, workspaceId, accessToken]);

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
