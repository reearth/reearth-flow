import { Dispatch, SetStateAction } from "react";
import * as Y from "yjs";

import { rebuildWorkflow } from "./conversions";
import type { YWorkflow } from "./types";
import useYEdge from "./useYEdge";
import useYHistory from "./useYHistory";
import useYLayout from "./useYLayout";
import useYNode from "./useYNode";
import useYWorkflow from "./useYWorkflow";

export default ({
  currentWorkflowId,
  yWorkflows,
  undoManager,
  globalWorkflowsUndoManager,
  setSelectedNodeIds,
  setSelectedEdgeIds,
  undoTrackerActionWrapper,
}: {
  currentWorkflowId: string;
  yWorkflows: Y.Map<YWorkflow>;
  undoManager: Y.UndoManager | null;
  globalWorkflowsUndoManager: Y.UndoManager | null;
  setSelectedNodeIds: Dispatch<SetStateAction<string[]>>;
  setSelectedEdgeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
}) => {
  const rawWorkflows = Array.from(yWorkflows.entries()).map(([_, yw]) =>
    rebuildWorkflow(yw),
  );

  // Wrap undoTrackerActionWrapper to automatically prepend current workflow ID
  const workflowScopedUndoWrapper = (
    callback: () => void,
    additionalPrefix?: string,
  ) => {
    const prefix = additionalPrefix
      ? `${currentWorkflowId}-${additionalPrefix}`
      : currentWorkflowId;
    undoTrackerActionWrapper(callback, prefix);
  };

  const {
    currentYWorkflow,
    handleYWorkflowAdd,
    handleYWorkflowUpdate,
    handleYWorkflowRemove,
    handleYWorkflowRename,
    handleYWorkflowAddFromSelection,
  } = useYWorkflow({
    yWorkflows,
    currentWorkflowId,
    undoTrackerActionWrapper: workflowScopedUndoWrapper,
  });

  const { handleYNodesAdd, handleYNodesChange, handleYNodeDataUpdate } =
    useYNode({
      currentYWorkflow,
      rawWorkflows,
      yWorkflows,
      setSelectedNodeIds,
      undoTrackerActionWrapper: workflowScopedUndoWrapper,
      handleYWorkflowRemove,
    });

  const { handleYEdgesAdd, handleYEdgesChange } = useYEdge({
    currentYWorkflow,
    setSelectedEdgeIds,
    undoTrackerActionWrapper: workflowScopedUndoWrapper,
  });

  const { canRedo, canUndo, handleYWorkflowRedo, handleYWorkflowUndo } =
    useYHistory({
      undoManager,
      globalWorkflowsUndoManager,
      undoTrackerActionWrapper,
    });

  const { handleYLayoutChange } = useYLayout({
    yWorkflows,
    rawWorkflows,
    undoTrackerActionWrapper: workflowScopedUndoWrapper,
  });

  return {
    canUndo,
    canRedo,
    rawWorkflows,
    currentYWorkflow,
    handleYWorkflowAdd,
    handleYWorkflowAddFromSelection,
    handleYWorkflowUpdate,
    handleYNodesAdd,
    handleYNodesChange,
    handleYNodeDataUpdate,
    handleYEdgesAdd,
    handleYEdgesChange,
    handleYWorkflowUndo,
    handleYWorkflowRedo,
    handleYWorkflowRename,
    handleYLayoutChange,
  };
};
