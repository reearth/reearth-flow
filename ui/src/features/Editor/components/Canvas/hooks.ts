import { DefaultEdgeOptions } from "@xyflow/react";
import { useCallback, useEffect, useState } from "react";

import { useYjsStore } from "@flow/lib/yjs";
import type { Node, Workflow } from "@flow/types";

import useEdges from "./useEdges";
import useNodes from "./useNodes";

type Props = {
  workflow?: Workflow;
  lockedNodeIds: string[];
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

export default ({ workflow, lockedNodeIds, onNodeLocking }: Props) => {
  const [currentWorkflowId, setCurrentWorkflowId] = useState<string>(workflow?.id ?? "");

  const { nodes, edges, handleNodesUpdate, handleEdgesUpdate } = useYjsStore();

  const processNode = useCallback(
    (node: Node) => {
      return {
        ...node,
        data: {
          ...node.data,
          locked: lockedNodeIds.includes(node.id),
          onLock: () => onNodeLocking(node.id, nodes, handleNodesUpdate),
        },
      };
    },
    [nodes, lockedNodeIds, onNodeLocking, handleNodesUpdate],
  );

  useEffect(() => {
    if (workflow && workflow.id !== currentWorkflowId) {
      handleNodesUpdate(workflow.nodes?.map(n => processNode(n)) ?? []);
      handleEdgesUpdate(workflow.edges ?? []);

      setCurrentWorkflowId(workflow.id);
    }
  }, [currentWorkflowId, workflow, processNode, handleNodesUpdate, handleEdgesUpdate]);

  const {
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDrop,
    handleNodeDragOver,
  } = useNodes({
    nodes,
    edges,
    onNodesChange: handleNodesUpdate,
    onEdgesChange: handleEdgesUpdate,
    onNodeLocking,
  });

  const { handleEdgesChange, handleConnect } = useEdges({
    edges,
    onEdgeChange: handleEdgesUpdate,
  });

  return {
    nodes,
    edges,
    handleNodesChange,
    handleNodesDelete,
    handleNodeDragStop,
    handleNodeDragOver,
    handleNodeDrop,
    handleEdgesChange,
    handleConnect,
  };
};
