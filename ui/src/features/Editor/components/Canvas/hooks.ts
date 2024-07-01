import { DefaultEdgeOptions } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback, useEffect, useState } from "react";

import { Edge, type Node, type Workflow } from "@flow/types";

import useEdges from "./useEdges";
import useNodes from "./useNodes";

type Props = {
  workflow?: Workflow;
  lockedNodeIds: string[];
  onNodeLocking: (nodeId: string, setNodes: Dispatch<SetStateAction<Node[]>>) => void;
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

  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>(workflow?.edges ?? []);

  const processNode = useCallback(
    (node: Node) => {
      return {
        ...node,
        data: {
          ...node.data,
          locked: lockedNodeIds.includes(node.id),
          onLock: () => onNodeLocking(node.id, setNodes),
        },
      };
    },
    [lockedNodeIds, setNodes, onNodeLocking],
  );

  useEffect(() => {
    if (workflow && workflow.id !== currentWorkflowId) {
      setNodes(workflow.nodes?.map(n => processNode(n)) ?? []);
      setEdges(workflow.edges ?? []);

      setCurrentWorkflowId(workflow.id);
    }
  }, [currentWorkflowId, workflow, processNode, setNodes, setEdges]);

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
    onNodeLocking,
  });

  const { handleEdgesChange, handleConnect } = useEdges({ setEdges });

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
