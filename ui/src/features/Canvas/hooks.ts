import {
  DefaultEdgeOptions,
  EdgeChange,
  NodeChange,
  XYPosition,
} from "@xyflow/react";
import { MouseEvent, useCallback, useRef, useState } from "react";

import type { ContextMenuMeta } from "@flow/components";
import { useEdges, useNodes } from "@flow/lib/reactFlow";
import type { ActionNodeType, Edge, Node } from "@flow/types";

type Props = {
  nodes: Node[];
  edges: Edge[];
  onWorkflowAdd?: (position?: XYPosition) => void;
  onNodesAdd?: (newNode: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onNodeDoubleClick?: (
    e: MouseEvent | undefined,
    nodeId: string,
    subworkflowId?: string,
  ) => void;
  onEdgesAdd?: (newEdges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onNodePickerOpen?: (position: XYPosition, nodeType?: ActionNodeType) => void;
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
  onNodeDoubleClick,
  onEdgesAdd,
  onEdgesChange,
  onNodePickerOpen,
}: Props) => {
  const {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragOver,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeDoubleClick,
  } = useNodes({
    nodes,
    edges,
    onWorkflowAdd,
    onNodesAdd,
    onNodesChange,
    onNodeDoubleClick,
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

  const handleNodeContextMenu = useCallback(
    (event: MouseEvent, node: Node) => {
      event.preventDefault();
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
      setContextMenu({
        node,
        styles,
      });
    },
    [setContextMenu],
  );

  const handleCloseContextmenu = () => {
    setContextMenu(null);
  };

  return {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleNodeDoubleClick,
    handleEdgesChange,
    handleConnect,
    handleReconnect,
    handleNodeContextMenu,
    handleCloseContextmenu,
    contextMenu,
    paneRef,
  };
};
