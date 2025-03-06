import { useReactFlow, XYPosition } from "@xyflow/react";
import { DragEvent, useCallback } from "react";

import {
  nodeTypes,
  type ActionNodeType,
  type Node,
  type NodeType,
} from "@flow/types";

import { buildNewCanvasNode } from "./buildNewCanvasNode";

type Props = {
  onWorkflowAdd?: (position?: XYPosition) => void;
  onNodesAdd?: (node: Node[]) => void;
  onNodePickerOpen?: (position: XYPosition, nodeType?: ActionNodeType) => void;
};

// This is used for drag and drop functionality in to the canvas
// This is not used for node dnd within the canvas. That is done internally by react-flow
export default ({ onWorkflowAdd, onNodesAdd, onNodePickerOpen }: Props) => {
  const { screenToFlowPosition } = useReactFlow();

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
        onWorkflowAdd?.(position);
        return;
      }

      if (
        nodeTypes.includes(type as NodeType) &&
        type !== "batch" &&
        type !== "note"
      ) {
        onNodePickerOpen?.(position, type as ActionNodeType);
        return;
      }

      const newNode = await buildNewCanvasNode({ position, type });

      if (!newNode) return;

      onNodesAdd?.([newNode]);
    },
    [screenToFlowPosition, onWorkflowAdd, onNodesAdd, onNodePickerOpen],
  );

  return { handleNodeDragOver, handleNodeDrop };
};
