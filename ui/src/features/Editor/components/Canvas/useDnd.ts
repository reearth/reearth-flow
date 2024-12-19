import { useReactFlow, XYPosition } from "@xyflow/react";
import { DragEvent, useCallback } from "react";

import {
  nodeTypes,
  type ActionNodeType,
  type Node,
  type NodeType,
} from "@flow/types";

import { useCreateNode } from "./useCreateNode";

type Props = {
  nodes: Node[];
  onWorkflowAdd: (position?: XYPosition) => void;
  onNodesChange: (nodes: Node[]) => void;
  onNodePickerOpen: (position: XYPosition, nodeType?: ActionNodeType) => void;
  handleNodeDropInBatch: (droppedNode: Node, nodes: Node[]) => Node[];
};

// This is used for drag and drop functionality in to the canvas
// This is not used for node dnd within the canvas. That is done internally by react-flow
export default ({
  nodes,
  onWorkflowAdd,
  onNodesChange,
  onNodePickerOpen,
  handleNodeDropInBatch,
}: Props) => {
  const { screenToFlowPosition } = useReactFlow();
  const { createNode } = useCreateNode();

  const handleNodeDragOver = useCallback((event: DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const handleNodeDrop = useCallback(
    async (event: DragEvent<HTMLDivElement>) => {
      event.preventDefault();

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });
      // Type can either be a node type or a action name
      const type = event.dataTransfer.getData("application/reactflow");
      if (!type) return;

      if (type === "subworkflow") {
        onWorkflowAdd(position);
        return;
      }

      if (
        nodeTypes.includes(type as NodeType) &&
        type !== "batch" &&
        type !== "note"
      ) {
        onNodePickerOpen(position, type as ActionNodeType);
        return;
      }

      const newNode = await createNode({ position, type });
      if (!newNode) return;

      const newNodes = [...nodes];
      // This is needed since children must be after the parent
      // in the array to avoid issues with react-flow
      if (type === "batch") {
        newNodes.splice(0, 0, newNode);
      } else {
        newNodes.push(newNode);
      }

      if (type !== "batch") {
        onNodesChange(handleNodeDropInBatch(newNode, newNodes));
      } else {
        onNodesChange(newNodes);
      }
    },
    [
      nodes,
      screenToFlowPosition,
      handleNodeDropInBatch,
      onWorkflowAdd,
      onNodesChange,
      onNodePickerOpen,
      createNode,
    ],
  );

  return { handleNodeDragOver, handleNodeDrop };
};
