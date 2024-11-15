import { DefaultEdgeOptions, XYPosition } from "@xyflow/react";

import type { ActionNodeType, Edge, Node, Workflow } from "@flow/types";

import useEdges from "./useEdges";
import useNodes from "./useNodes";

type Props = {
  workflow?: Workflow;
  nodes: Node[];
  edges: Edge[];
  onWorkflowAdd: (position?: XYPosition) => void;
  onNodesUpdate: (newNodes: Node[]) => void;
  onEdgesUpdate: (newEdges: Edge[]) => void;
  onNodeLocking: (nodeId: string) => void;
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
  onNodeLocking,
  onNodesUpdate,
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
    onNodesChange: onNodesUpdate,
    onEdgesChange: onEdgesUpdate,
    onNodeLocking,
    onNodePickerOpen,
  });

  const { handleEdgesChange, handleConnect, handleReconnect } = useEdges({
    edges,
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
