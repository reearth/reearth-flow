import { NodeChange, useReactFlow, XYPosition } from "@xyflow/react";
import { DragEvent, useCallback } from "react";

import {
  nodeTypes,
  type ActionNodeType,
  type Node,
  type NodeType,
} from "@flow/types";

import { useT } from "../i18n";

import { buildNewCanvasNode } from "./buildNewCanvasNode";

type Props = {
  nodes: Node[];
  selectedNodeIds?: string[];
  onWorkflowAdd?: (position?: XYPosition) => void;
  onNodesAdd?: (node: Node[]) => void;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onNodePickerOpen?: (position: XYPosition, nodeType?: ActionNodeType) => void;
};

// This is used for drag and drop functionality in to the canvas
// This is not used for node dnd within the canvas. That is done internally by react-flow
export default ({
  nodes,
  selectedNodeIds,
  onWorkflowAdd,
  onNodesAdd,
  onNodesChange,
  onNodePickerOpen,
}: Props) => {
  const t = useT();
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

      const officialName =
        type === "batch" ? t("Batch") : type === "note" ? t("Note") : type;

      const newNode = await buildNewCanvasNode({
        position,
        type,
        officialName,
      });

      if (newNode?.type === "batch" || newNode?.type === "note") {
        const selectedNodes = nodes.filter((n) =>
          selectedNodeIds?.includes(n.id),
        );

        if (selectedNodes.length) {
          const nodesToDeselect: NodeChange<Node>[] = selectedNodes.map(
            (node) => ({
              type: "select",
              id: node.id,
              selected: false,
            }),
          );
          onNodesChange?.(nodesToDeselect);
        }
      }

      if (!newNode) return;

      onNodesAdd?.([newNode]);
    },
    [
      t,
      nodes,
      selectedNodeIds,
      screenToFlowPosition,
      onWorkflowAdd,
      onNodesAdd,
      onNodesChange,
      onNodePickerOpen,
    ],
  );

  return { handleNodeDragOver, handleNodeDrop };
};
