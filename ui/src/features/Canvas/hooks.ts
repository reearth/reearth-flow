import {
  DefaultEdgeOptions,
  EdgeChange,
  NodeChange,
  XYPosition,
} from "@xyflow/react";
import { MouseEvent, useCallback, useRef, useState } from "react";
import { useHotkeys } from "react-hotkeys-hook";

import type { ContextMenuMeta } from "@flow/components";
import { CANVAS_HOT_KEYS } from "@flow/global-constants";
import { useEdges, useNodes } from "@flow/lib/reactFlow";
import type { ActionNodeType, Edge, Node } from "@flow/types";

type Props = {
  nodes: Node[];
  edges: Edge[];
  onWorkflowAdd?: (position?: XYPosition) => void;
  onNodesAdd?: (newNode: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onNodeSettings?: (e: MouseEvent | undefined, nodeId: string) => void;
  onEdgesAdd?: (newEdges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onNodePickerOpen?: (
    position: XYPosition,
    nodeType?: ActionNodeType,
    isMainWorkflow?: boolean,
  ) => void;
  onCopy?: (node?: Node) => void;
  onCut?: (isCutByShortCut?: boolean, node?: Node) => void;
  onPaste?: () => void;
};

export const defaultEdgeOptions: DefaultEdgeOptions = {
  // stroke style for unsure (normal) state: rgb(234, 179, 8) bg-yellow-500
  // stroke style for success state: rgb(22, 163, 74) bg-green (after running workflow)
  // stroke style for error state: "#7f1d1d" (after running workflow)
  // style: { strokeWidth: 2, stroke: "rgb(234, 179, 8)" },
  // type: "floating",
  //   markerEnd: {
  //     type: MarkerType.ArrowClosed,
  //     color: "red",
  //   },
  //   markerStart: {
  //     type: MarkerType.ArrowClosed,
  //     color: "green",
  //   },
  // animated: true,
};

export default ({
  nodes,
  edges,
  onWorkflowAdd,
  onNodesAdd,
  onNodesChange,
  onNodeSettings,
  onEdgesAdd,
  onEdgesChange,
  onNodePickerOpen,
  onCopy,
  onCut,
  onPaste,
}: Props) => {
  const {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragOver,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeSettings,
  } = useNodes({
    nodes,
    edges,
    onWorkflowAdd,
    onNodesAdd,
    onNodesChange,
    onNodeSettings,
    onEdgesChange,
    onNodePickerOpen,
  });

  const { handleEdgesChange, handleConnect, handleReconnect } = useEdges({
    edges,
    onEdgesAdd,
    onEdgesChange,
  });

  const [contextMenu, setContextMenu] = useState<ContextMenuMeta | null>(null);
  const paneRef = useRef<HTMLDivElement>(null);
  const getContextMenuPosition = (event: MouseEvent) => {
    if (!paneRef.current) return;
    const pane = paneRef.current.getBoundingClientRect();
    const localX = event.clientX - pane.left;
    const localY = event.clientY - pane.top;
    const styles: React.CSSProperties = {};
    if (localY < pane.height - 200) {
      styles.top = localY;
    } else {
      styles.bottom = pane.height - localY;
    }

    if (localX < pane.width - 200) {
      styles.left = localX;
    } else {
      styles.right = pane.width - localX;
    }
    return { styles, mousePosition: { x: localX, y: localY } };
  };

  const handleNodeContextMenu = useCallback(
    (event: MouseEvent, node: Node) => {
      event.preventDefault();
      const position = getContextMenuPosition(event);
      if (!position) return;
      const { styles } = position;

      setContextMenu({
        data: node,
        styles,
      });
    },
    [setContextMenu],
  );

  const handleSelectionContextMenu = useCallback(
    (event: MouseEvent, nodes: Node[]) => {
      event.preventDefault();
      const position = getContextMenuPosition(event);
      if (!position) return;
      const { styles } = position;

      setContextMenu({
        data: nodes,
        styles,
      });
    },
    [setContextMenu],
  );

  const handlePaneContextMenu = useCallback(
    (event: MouseEvent | globalThis.MouseEvent) => {
      event.preventDefault();
      const position = getContextMenuPosition(event as MouseEvent);
      if (!position) return;
      const { styles, mousePosition } = position;
      setContextMenu({
        mousePosition,
        styles,
      });
    },
    [setContextMenu],
  );

  const handleCloseContextmenu = () => {
    setContextMenu(null);
  };

  useHotkeys(CANVAS_HOT_KEYS, (event, handler) => {
    const hasModifier = event.metaKey || event.ctrlKey;
    switch (handler.keys?.join("")) {
      case "r":
        event.preventDefault();
        onNodePickerOpen?.({ x: 0, y: 0 }, "reader", true);
        break;
      case "t":
        event.preventDefault();
        onNodePickerOpen?.({ x: 0, y: 0 }, "transformer");
        break;
      case "w":
        event.preventDefault();
        onNodePickerOpen?.({ x: 0, y: 0 }, "writer", true);
        break;
      case "c":
        if (hasModifier) onCopy?.();
        break;
      case "x":
        if (hasModifier) onCut?.();
        break;
      case "v":
        if (hasModifier) onPaste?.();
        break;
    }
  });

  return {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleNodeSettings,
    handleEdgesChange,
    handleConnect,
    handleReconnect,
    handleNodeContextMenu,
    handleSelectionContextMenu,
    handlePaneContextMenu,
    handleCloseContextmenu,
    contextMenu,
    paneRef,
  };
};
