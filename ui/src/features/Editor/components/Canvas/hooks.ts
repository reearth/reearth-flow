import {
  DefaultEdgeOptions,
  EdgeChange,
  NodeChange,
  useReactFlow,
  XYPosition,
} from "@xyflow/react";
import { useEffect } from "react";

import type { ActionNodeType, Edge, Node } from "@flow/types";

import useEdges from "./useEdges";
import useNodes from "./useNodes";

type Props = {
  nodes: Node[];
  edges: Edge[];
  onWorkflowAdd: (position?: XYPosition) => void;
  onNodesAdd: (newNode: Node[]) => void;
  onNodesChange: (changes: NodeChange<Node>[]) => void;
  onEdgesAdd: (newEdges: Edge[]) => void;
  onEdgesChange: (changes: EdgeChange[]) => void;
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
  onNodesAdd,
  onNodesChange,
  onEdgesAdd,
  onEdgesChange,
  onNodePickerOpen,
}: Props) => {
  const { fitView } = useReactFlow();

  // Fit all nodes into view on mount
  useEffect(() => {
    fitView({ padding: 0.2 });
  }, []); // eslint-disable-line

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
    onNodesAdd,
    onNodesChange,
    onEdgesChange,
    onNodePickerOpen,
  });

  const { handleEdgesChange, handleConnect, handleReconnect } = useEdges({
    onEdgesAdd,
    onEdgesChange,
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
