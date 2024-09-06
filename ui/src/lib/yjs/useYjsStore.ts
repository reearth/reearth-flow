import { useCallback, useEffect, useRef, useState } from "react";
import { useY } from "react-yjs";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import type { Edge, Node } from "@flow/types";

import useWorkflowTabs from "./useWorkflowTabs";
import useYEdge from "./useYEdge";
import useYNode from "./useYNode";
import useYWorkflow from "./useYWorkflow";
import { yWorkflowBuilder, type YWorkflow } from "./workflowBuilder";

class CustomBinding {} // eslint-disable-line @typescript-eslint/no-extraneous-class

export default ({
  workflowId,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const yWebSocketRef = useRef<WebsocketProvider | null>(null);
  useEffect(() => () => yWebSocketRef.current?.destroy(), []);

  const [undoManager, setUndoManager] = useState<Y.UndoManager | null>(null);

  const [{ yWorkflows, currentUserClientId, undoTrackerActionWrapper }] =
    useState(() => {
      const yDoc = new Y.Doc();
      const { websocket, websocketToken } = config();
      if (workflowId && websocket && websocketToken) {
        yWebSocketRef.current = new WebsocketProvider(
          websocket,
          workflowId,
          yDoc,
          { params: { token: websocketToken } },
        );
      }

      const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
      const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
      yWorkflows.push([yWorkflow]);

      const currentUserClientId = yDoc.clientID;

      // NOTE: any changes to the yDoc should be wrapped in a transact
      const undoTrackerActionWrapper = (callback: () => void) =>
        yDoc.transact(callback, CustomBinding);

      return { yWorkflows, currentUserClientId, undoTrackerActionWrapper };
    });

  useEffect(() => {
    if (yWorkflows) {
      const manager = new Y.UndoManager(yWorkflows, {
        trackedOrigins: new Set([currentUserClientId, CustomBinding]), // Only track local changes
      });
      setUndoManager(manager);

      return () => {
        manager.destroy(); // Clean up UndoManager on component unmount
      };
    }
  }, [yWorkflows, currentUserClientId]);

  const handleWorkflowUndo = useCallback(
    () => undoManager?.undo(),
    [undoManager],
  );

  const handleWorkflowRedo = useCallback(
    () => undoManager?.redo(),
    [undoManager],
  );

  const rawWorkflows = useY(yWorkflows);

  const {
    workflows,
    openWorkflows,
    currentWorkflowIndex,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  } = useWorkflowTabs({ workflowId, rawWorkflows, handleWorkflowIdChange });

  const { currentYWorkflow, handleWorkflowAdd, handleWorkflowsRemove } =
    useYWorkflow({
      yWorkflows,
      workflows,
      currentWorkflowIndex,
      undoTrackerActionWrapper,
      setWorkflows,
      setOpenWorkflowIds,
      handleWorkflowIdChange,
      handleWorkflowOpen,
    });

  const nodes = useY(
    currentYWorkflow?.get("nodes") ?? new Y.Array<Node>(),
  ) as Node[];
  const edges = useY(
    currentYWorkflow?.get("edges") ?? new Y.Array<Edge>(),
  ) as Edge[];

  const { handleNodesUpdate } = useYNode({
    currentYWorkflow,
    undoTrackerActionWrapper,
    handleWorkflowsRemove,
  });

  const { handleEdgesUpdate } = useYEdge({
    currentYWorkflow,
    undoTrackerActionWrapper,
  });

  return {
    nodes,
    edges,
    openWorkflows,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleNodesUpdate,
    handleEdgesUpdate,
    handleWorkflowUndo,
    handleWorkflowRedo,
  };
};
