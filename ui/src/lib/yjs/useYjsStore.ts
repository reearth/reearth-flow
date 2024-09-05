import { useEffect, useRef, useState } from "react";
import { useY } from "react-yjs";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import useWorkflowTabs from "./useWorkflowTabs";
import useYEdge from "./useYEdge";
import useYNode from "./useYNode";
import useYWorkflow from "./useYWorkflow";
import { yWorkflowBuilder, type YWorkflow } from "./workflowBuilder";

export default ({
  workflowId,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const [undoManager, setUndoManager] = useState<Y.UndoManager | null>(null);
  const yWebSocketRef = useRef<WebsocketProvider | null>(null);
  useEffect(() => () => yWebSocketRef.current?.destroy(), []);

  const [{ yWorkflows, currentUserClientId, undoTrackerActionWrapper }] =
    useState(() => {
      const yDoc = new Y.Doc();
      yWebSocketRef.current = new WebsocketProvider(
        "ws://localhost:8000",
        workflowId ? workflowId : "",
        yDoc,
        { params: { token: "nyaan" } },
      );

      const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
      const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
      yWorkflows.push([yWorkflow]);

      const currentUserClientId = yDoc?.clientID;

      // const undoManager = new Y.UndoManager(yWorkflows, {
      //   trackedOrigins: new Set([currentUserClientId]), // Only track local changes
      // });

      // setUndoManager(undoManager);

      // NOTE: any changes to the yDoc should be wrapped in a transact
      const undoTrackerActionWrapper = (callback: () => void) =>
        yDoc.transact(callback);

      return { yWorkflows, currentUserClientId, undoTrackerActionWrapper };
    });

  useEffect(() => {
    if (yWorkflows) {
      // Now that yWorkflow is set, create the UndoManager
      const manager = new Y.UndoManager(yWorkflows, {
        trackedOrigins: new Set([currentUserClientId]), // Only track local changes
      });
      setUndoManager(manager);

      return () => {
        manager.destroy(); // Clean up UndoManager on component unmount
      };
    }
  }, [yWorkflows, currentUserClientId]);

  const undo = () => {
    undoManager?.undo();
    console.log("Undo stack size:", undoManager?.undoStack.length);
    console.log("Redo stack size:", undoManager?.redoStack.length);
  };
  const redo = () => {
    console.log("REDO");
    undoManager?.redo();
    console.log("Undo stack size:", undoManager?.undoStack.length);
    console.log("Redo stack size:", undoManager?.redoStack.length);
  };

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

  const { handleEdgesUpdate } = useYEdge(currentYWorkflow);

  return {
    nodes,
    edges,
    openWorkflows,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleNodesUpdate,
    handleEdgesUpdate,
    handleWorkflowUndo: undo,
    handleWorkflowRedo: redo,
  };
};
