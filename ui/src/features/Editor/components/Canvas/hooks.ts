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
import { MouseEvent, useEffect, useState } from "react";

import { Node, Workflow } from "@flow/types";

import useBatch from "./useBatch";
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
  const { isNodeIntersecting } = useReactFlow();
  // console.log("reactFlowInstance", toObject());
  const [currentWorkflowId, setCurrentWorkflowId] = useState<string>(workflow?.id ?? "");

  const [nodes, setNodes] = useState(workflow?.nodes ?? []);
  const [edges, setEdges] = useState(workflow?.edges ?? []);

  const { onDragOver, onDrop } = useDnd({ setNodes });
  const { handleAddToBatch, handleRemoveFromBatch } = useBatch();

  useEffect(() => {
    if (workflow && workflow.id !== currentWorkflowId) {
      setNodes(workflow.nodes ?? []);
      setEdges(workflow.edges ?? []);

      setCurrentWorkflowId(workflow.id);
    }
  }, [currentWorkflowId, workflow, setNodes, setEdges]);

  const onNodesChange: OnNodesChange<Node> = changes => {
    setNodes(nds => applyNodeChanges<Node>(changes, nds));
  };

  const onEdgesChange: OnEdgesChange = changes => setEdges(eds => applyEdgeChanges(changes, eds));

  const onConnect: OnConnect = connection => setEdges(eds => addEdge(connection, eds));

  const onNodeDragStop = (_evt: MouseEvent, node: Node) => {
    if (node.type === "batch") {
      return;
    }
    nodes.forEach(nd => {
      if (nd.type === "batch") {
        //safety check to make sure there's a height and width
        if (nd.measured?.height && nd.measured?.width) {
          const rec = {
            height: nd.measured.height,
            width: nd.measured.width,
            ...nd.position,
          };

          // Check if the dragged node is inside the group
          if (isNodeIntersecting(node, rec, false)) {
            handleAddToBatch(node, nd, setNodes);
          } else {
            handleRemoveFromBatch(node, nd, setNodes);
          }
        }
      }
    });
  };

  return {
    nodes,
    edges,
    onDragOver,
    onDrop,
    onNodesChange,
    onNodeDragStop,
    onEdgesChange,
    onConnect,
  };
};
