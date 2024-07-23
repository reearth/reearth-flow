import { DefaultEdgeOptions } from "@xyflow/react";

import type { Edge, Node, Workflow } from "@flow/types";

import useEdges from "./useEdges";
import useNodes from "./useNodes";

type Props = {
  workflow?: Workflow;
  nodes: Node[];
  edges: Edge[];
  onNodesUpdate: (newNodes: Node[]) => void;
  onEdgesUpdate: (newEdges: Edge[]) => void;
  onNodeLocking: (nodeId: string, nodes: Node[], onNodesChange: (nodes: Node[]) => void) => void;
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

export default ({ nodes, edges, onNodeLocking, onNodesUpdate, onEdgesUpdate }: Props) => {
  const {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeDragOver,
  } = useNodes({
    nodes,
    edges,
    onNodesChange: onNodesUpdate,
    onEdgesChange: onEdgesUpdate,
    onNodeLocking,
  });

  const { handleEdgesChange, handleConnect } = useEdges({
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
  };
};
