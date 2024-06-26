import { DefaultEdgeOptions } from "@xyflow/react";
import { useEffect, useState } from "react";

import type { Workflow } from "@flow/types";

import useEdges from "./useEdges";
import useNodes from "./useNodes";

type Props = {
  workflow?: Workflow;
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

export default ({ workflow }: Props) => {
  const [currentWorkflowId, setCurrentWorkflowId] = useState<string>(workflow?.id ?? "");

  const [nodes, setNodes] = useState(workflow?.nodes ?? []);
  const [edges, setEdges] = useState(workflow?.edges ?? []);

  useEffect(() => {
    if (workflow && workflow.id !== currentWorkflowId) {
      setNodes(workflow.nodes ?? []);
      setEdges(workflow.edges ?? []);

      setCurrentWorkflowId(workflow.id);
    }
  }, [currentWorkflowId, workflow, setNodes, setEdges]);

  const {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeDragOver,
  } = useNodes({
    nodes,
    edges,
    setNodes,
    setEdges,
  });

  const { handleEdgesChange, handleConnect } = useEdges({ setEdges });

  // const { softSelect, hardSelect, useSingleClick, useDoubleClick } = useSelect();

  return {
    nodes,
    edges,
    // softSelect,
    // hardSelect,
    // useSingleClick,
    // useDoubleClick,
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleEdgesChange,
    handleConnect,
  };
};
