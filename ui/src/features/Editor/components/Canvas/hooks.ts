import { DefaultEdgeOptions, NodeChange, XYPosition } from "@xyflow/react";

import type { ActionNodeType, Edge, Node } from "@flow/types";

import useEdges from "./useEdges";
import useNodes from "./useNodes";

type Props = {
  nodes: Node[];
  edges: Edge[];
  onWorkflowAdd: (position?: XYPosition) => void;
  onNodeSelection: (selecting: string[], deselecting: string[]) => void;
  onNodesChange2: (changes: NodeChange<Node>[]) => void;
  onNodesUpdate: (newNodes: Node[]) => void;
  onEdgeSelection: (idsToAdd: string[], idsToDelete: string[]) => void;
  onEdgesUpdate: (newEdges: Edge[]) => void;
  onNodePickerOpen: (position: XYPosition, nodeType?: ActionNodeType) => void;
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
  onNodesUpdate,
  onNodesChange2,
  onNodeSelection,
  onEdgeSelection,
  onEdgesUpdate,
  onNodePickerOpen,
}: Props) => {
  const {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeDragOver,
  } = useNodes({
    nodes,
    edges,
    onWorkflowAdd,
    onNodeSelection,
    onNodesChange: onNodesUpdate,
    onNodesChange2,
    onEdgesChange: onEdgesUpdate,
    onNodePickerOpen,
  });

  const { handleEdgesChange, handleConnect, handleReconnect } = useEdges({
    edges,
    onEdgeSelection,
    onEdgeChange: onEdgesUpdate,
  });

  return {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleEdgesChange,
    handleConnect,
    handleReconnect,
  };
};
