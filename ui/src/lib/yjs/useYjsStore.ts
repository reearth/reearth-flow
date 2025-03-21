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
  setSelectedNodeIds,
  setSelectedEdgeIds,
  undoTrackerActionWrapper,
}: {
  currentWorkflowId: string;
  yWorkflows: Y.Map<YWorkflow>;
  undoManager: Y.UndoManager | null;
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
    undoTrackerActionWrapper,
  });

  const { handleYNodesAdd, handleYNodesChange, handleYNodeDataUpdate } =
    useYNode({
      currentYWorkflow,
      rawWorkflows,
      yWorkflows,
      setSelectedNodeIds,
      undoTrackerActionWrapper,
      handleYWorkflowRemove,
    });

  const { handleYEdgesAdd, handleYEdgesChange } = useYEdge({
    currentYWorkflow,
    setSelectedEdgeIds,
    undoTrackerActionWrapper,
  });

  const { canRedo, canUndo, handleYWorkflowRedo, handleYWorkflowUndo } =
    useYHistory({ undoManager, undoTrackerActionWrapper });

  const { handleYLayoutChange } = useYLayout({
    yWorkflows,
    rawWorkflows,
    undoTrackerActionWrapper,
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
