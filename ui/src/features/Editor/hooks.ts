import { XYPosition } from "@xyflow/react";
import { MouseEvent, useCallback, useState } from "react";

import { useShortcuts } from "@flow/hooks";
import { useYjsStore } from "@flow/lib/yjs";
import { useCurrentWorkflowId } from "@flow/stores";
import type { ActionNodeType, Edge, Node } from "@flow/types";
import { cancellableDebounce } from "@flow/utils";

import useCanvasCopyPaste from "./useCanvasCopyPaste";
import useNodeLocker from "./useNodeLocker";

export default () => {
  const [currentWorkflowId, setCurrentWorkflowId] = useCurrentWorkflowId();

  const handleWorkflowIdChange = useCallback(
    (id?: string) => {
      if (!id) return setCurrentWorkflowId(undefined);
      setCurrentWorkflowId(id);
    },
    [setCurrentWorkflowId],
  );

  const {
    openWorkflows,
    nodes,
    edges,
    handleWorkflowDeployment,
    handleWorkflowAdd,
    handleWorkflowClose,
    handleNodesUpdate,
    handleEdgesUpdate,
    handleWorkflowRedo,
    handleWorkflowUndo,
    handleWorkflowRename,
  } = useYjsStore({
    workflowId: currentWorkflowId,
    handleWorkflowIdChange,
  });

  const { lockedNodeIds, locallyLockedNode, handleNodeLocking } = useNodeLocker(
    { handleNodesUpdate },
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
    handleWorkflowDeployment,
    handlePanelOpen,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleWorkflowChange: handleWorkflowIdChange,
    handleNodesUpdate,
    handleNodeHover,
    handleNodeLocking,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesUpdate,
    handleEdgeHover,
    handleWorkflowRedo,
    handleWorkflowUndo,
    handleWorkflowRename,
  };
};
