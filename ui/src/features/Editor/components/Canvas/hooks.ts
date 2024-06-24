import {
  DefaultEdgeOptions,
  OnConnect,
  OnEdgesChange,
  OnNodesChange,
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  useReactFlow,
} from "@xyflow/react";
import { useEffect, useState } from "react";

import { Node, Workflow } from "@flow/types";

import useDnd from "./useDnd";

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
  const reactFlowInstance = useReactFlow();
  console.log("reactFlowInstance", reactFlowInstance.toObject());
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

  const { onDragOver, onDrop } = useDnd({ setNodes });

  const onNodesChange: OnNodesChange<Node> = changes => {
    setNodes(nds => applyNodeChanges<Node>(changes, nds));
  };

  const onEdgesChange: OnEdgesChange = changes => setEdges(eds => applyEdgeChanges(changes, eds));

  const onConnect: OnConnect = connection => setEdges(eds => addEdge(connection, eds));

  // useEffect(() => {
  //   if (workflow) {
  //     setNodes(workflow.nodes ?? []);
  //     setEdges(workflow.edges ?? []);
  //   }
  // }, [workflow, setNodes, setEdges]);

  return {
    nodes,
    edges,
    onDragOver,
    onDrop,
    onNodesChange,
    onEdgesChange,
    onConnect,
  };
};
