import { XYPosition } from "@xyflow/react";
import { MouseEvent, useCallback, useState } from "react";
import { Array as YArray, UndoManager as YUndoManager } from "yjs";

import { useShortcuts } from "@flow/hooks";
import { useYjsStore } from "@flow/lib/yjs";
import { YWorkflow } from "@flow/lib/yjs/utils";
import { useCurrentWorkflowId } from "@flow/stores";
import type { ActionNodeType, Edge, Node } from "@flow/types";
import { cancellableDebounce } from "@flow/utils";

import useCanvasCopyPaste from "./useCanvasCopyPaste";
import useNodeLocker from "./useNodeLocker";

export default ({
  yWorkflows,
  undoManager,
  undoTrackerActionWrapper,
}: {
  yWorkflows: YArray<YWorkflow>;
  undoManager: YUndoManager | null;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const [currentWorkflowId, setCurrentWorkflowId] = useCurrentWorkflowId();

  const handleWorkflowIdChange = useCallback(
    (id?: string) => {
      if (!id) return setCurrentWorkflowId(undefined);
      setCurrentWorkflowId(id);
    },
    [setCurrentWorkflowId],
  );

  const {
    nodes,
    edges,
    openWorkflows,
    selectedNodes,
    handleWorkflowDeployment,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleNodesUpdate,
    handleEdgesUpdate,
    handleWorkflowUndo,
    handleWorkflowRedo,
    handleWorkflowRename,
  } = useYjsStore({
    workflowId: currentWorkflowId,
    yWorkflows,
    undoManager,
    undoTrackerActionWrapper,
    handleWorkflowIdChange,
  });

  const { lockedNodeIds, locallyLockedNode, handleNodeLocking } = useNodeLocker(
    { selectedNodes, handleNodesUpdate },
  );

  const handleNodeDoubleClick = useCallback(
    (_e: MouseEvent, node: Node) => {
      if (node.type === "subworkflow") {
        handleWorkflowOpen(node.id);
      } else {
        handleNodeLocking(node.id);
      }
    },
    [handleWorkflowOpen, handleNodeLocking],
  );

  const { handleCopy, handlePaste } = useCanvasCopyPaste({
    nodes,
    edges,
    handleNodesUpdate,
    handleEdgesUpdate,
  });

  const [openPanel, setOpenPanel] = useState<
    "left" | "right" | "bottom" | undefined
  >(undefined);

  const handlePanelOpen = useCallback(
    (panel?: "left" | "right" | "bottom") => {
      if (!panel || openPanel === panel) {
        setOpenPanel(undefined);
      } else {
        setOpenPanel(panel);
      }
    },
    [openPanel],
  );

  const [nodePickerOpen, setNodePickerOpen] = useState<
    { position: XYPosition; nodeType: ActionNodeType } | undefined
  >(undefined);

  const handleNodePickerOpen = useCallback(
    (position?: XYPosition, nodeType?: ActionNodeType) => {
      setNodePickerOpen(
        !position || !nodeType ? undefined : { position, nodeType },
      );
    },
    [],
  );

  const handleNodePickerClose = useCallback(
    () => setNodePickerOpen(undefined),
    [],
  );

  const [hoveredDetails, setHoveredDetails] = useState<
    Node | Edge | undefined
  >();

  const hoverActionDebounce = cancellableDebounce(
    (callback: () => void) => callback(),
    100,
  );

  const handleNodeHover = useCallback(
    (e: MouseEvent, node?: Node) => {
      hoverActionDebounce.cancel();
      if (e.type === "mouseleave" && hoveredDetails) {
        hoverActionDebounce(() => setHoveredDetails(undefined));
      } else {
        setHoveredDetails(node);
      }
    },
    [hoveredDetails, hoverActionDebounce],
  );

  const handleEdgeHover = useCallback(
    (e: MouseEvent, edge?: Edge) => {
      if (e.type === "mouseleave" && hoveredDetails) {
        setHoveredDetails(undefined);
      } else {
        setHoveredDetails(edge);
      }
    },
    [hoveredDetails],
  );

  useShortcuts([
    {
      keyBinding: { key: "r", commandKey: false },
      callback: () => handleNodePickerOpen({ x: 0, y: 0 }, "reader"),
    },
    {
      keyBinding: { key: "t", commandKey: false },
      callback: () => handleNodePickerOpen({ x: 0, y: 0 }, "transformer"),
    },
    {
      keyBinding: { key: "w", commandKey: false },
      callback: () => handleNodePickerOpen({ x: 0, y: 0 }, "writer"),
    },
    {
      keyBinding: { key: "c", commandKey: true },
      callback: handleCopy,
    },
    {
      keyBinding: { key: "v", commandKey: true },
      callback: handlePaste,
    },
    {
      keyBinding: { key: "z", commandKey: true, shiftKey: true },
      callback: handleWorkflowRedo,
    },
    {
      keyBinding: { key: "z", commandKey: true },
      callback: handleWorkflowUndo,
    },
  ]);

  return {
    currentWorkflowId,
    openWorkflows,
    nodes,
    edges,
    lockedNodeIds,
    locallyLockedNode,
    hoveredDetails,
    nodePickerOpen,
    openPanel,
    handleWorkflowAdd,
    handleWorkflowDeployment,
    handlePanelOpen,
    handleWorkflowClose,
    handleWorkflowChange: handleWorkflowIdChange,
    handleNodesUpdate,
    handleNodeHover,
    handleNodeDoubleClick,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesUpdate,
    handleEdgeHover,
    handleWorkflowRedo,
    handleWorkflowUndo,
    handleWorkflowRename,
  };
};
