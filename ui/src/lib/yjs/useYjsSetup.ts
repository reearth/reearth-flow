import { useParams } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { YWorkflow, yWorkflowBuilder } from "./utils";

export default ({ workflowId }: { workflowId?: string }) => {
  const { projectId }: { projectId: string } = useParams({
    strict: false,
  });

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
      yWebSocketProvider = new WebsocketProvider(
        websocket,
        `${projectId}:${workflowId}`,
        yDoc,
      );

      yWebSocketProvider.once("sync", () => {
        if (yWorkflows.length === 0) {
          yDoc.transact(() => {
            const yWorkflow = yWorkflowBuilder(
              DEFAULT_ENTRY_GRAPH_ID,
              "Main Workflow",
            );
            yWorkflows.push([yWorkflow]);
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
      yWebSocketProvider?.destroy(); // Cleanup on unmount
    };
  }, [projectId, workflowId]);

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
