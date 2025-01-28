import { Dispatch, SetStateAction } from "react";
import * as Y from "yjs";

import { rebuildWorkflow } from "./conversions";
import type { YWorkflow } from "./types";
import useYEdge from "./useYEdge";
import useYHistory from "./useYHistory";
import useYNode from "./useYNode";
import useYWorkflow from "./useYWorkflow";

export default ({
  currentWorkflowId,
  yWorkflows,
  undoManager,
  setSelectedNodeIds,
  undoTrackerActionWrapper,
}: {
  currentWorkflowId: string;
  yWorkflows: Y.Array<YWorkflow>;
  undoManager: Y.UndoManager | null;
  setSelectedNodeIds: Dispatch<SetStateAction<string[]>>;
  setSelectedEdgeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const rawWorkflows = yWorkflows.map((w) => rebuildWorkflow(w));

  const {
    currentYWorkflow,
    handleYWorkflowAdd,
    handleYWorkflowUpdate,
    handleYWorkflowsRemove,
    handleYWorkflowRename,
    handleYWorkflowAddFromSelection,
  } = useYWorkflow({
    yWorkflows,
    rawWorkflows,
    currentWorkflowId,
    undoTrackerActionWrapper,
  });

  const { handleYNodesAdd, handleYNodesChange, handleYNodeParamsUpdate } =
    useYNode({
      currentYWorkflow,
      rawWorkflows,
      yWorkflows,
      setSelectedNodeIds,
      undoTrackerActionWrapper,
      handleWorkflowsRemove: handleYWorkflowsRemove,
    });

  const { handleYEdgesUpdate } = useYEdge({
    currentYWorkflow,
    undoTrackerActionWrapper,
  });

  const { canRedo, canUndo, handleYWorkflowRedo, handleYWorkflowUndo } =
    useYHistory({ undoManager });

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
    handleYNodeParamsUpdate,
    handleYEdgesUpdate,
    handleYWorkflowUndo,
    handleYWorkflowRedo,
    handleYWorkflowRename,
  };
};
