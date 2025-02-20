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
  yWorkflows: Y.Array<YWorkflow>;
  undoManager: Y.UndoManager | null;
  setSelectedNodeIds: Dispatch<SetStateAction<string[]>>;
  setSelectedEdgeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const rawWorkflows = yWorkflows.map((w) => rebuildWorkflow(w));

  console.log("rawWorkflows", rawWorkflows);

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
      handleYWorkflowsRemove,
    });

  const { handleYEdgesAdd, handleYEdgesChange } = useYEdge({
    currentYWorkflow,
    setSelectedEdgeIds,
    undoTrackerActionWrapper,
  });

  const { canRedo, canUndo, handleYWorkflowRedo, handleYWorkflowUndo } =
    useYHistory({ undoManager });

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
    handleYNodeParamsUpdate,
    handleYEdgesAdd,
    handleYEdgesChange,
    handleYWorkflowUndo,
    handleYWorkflowRedo,
    handleYWorkflowRename,
    handleYLayoutChange,
  };
};
