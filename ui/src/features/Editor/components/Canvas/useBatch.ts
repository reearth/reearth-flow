import { useReactFlow } from "@xyflow/react";
import { Dispatch, SetStateAction } from "react";

import { Node } from "@flow/types";

export default () => {
  const { getInternalNode } = useReactFlow();

  const handleAddToBatch = (
    draggedNode: Node,
    hoveredNode: Node,
    setNodes: Dispatch<SetStateAction<Node[]>>,
  ) => {
    // Check if dragged node isn't already a child to the group
    if (!draggedNode.parentId) {
      draggedNode.parentId = hoveredNode.id;
      const posX = getInternalNode(draggedNode.id)?.position.x;
      const posY = getInternalNode(draggedNode.id)?.position.y;
      if (posX && posY) {
        draggedNode.position = {
          x: posX - hoveredNode.position.x,
          y: posY - hoveredNode.position.y,
        };
      }
      setNodes(nodes =>
        nodes.map(n => {
          if (n.id === draggedNode.id) {
            n = draggedNode;
          }
          return n;
        }),
      );
    }
  };

  const handleRemoveFromBatch = (
    draggedNode: Node,
    hoveredNode: Node,
    setNodes: Dispatch<SetStateAction<Node[]>>,
  ) => {
    // Check if dragged node is a child to the group
    if (draggedNode.parentId === hoveredNode.id) {
      draggedNode.parentId = undefined;
      const posX = getInternalNode(draggedNode.id)?.position.x;
      const posY = getInternalNode(draggedNode.id)?.position.y;
      if (posX && posY) {
        draggedNode.position = {
          x: posX + hoveredNode.position.x,
          y: posY + hoveredNode.position.y,
        };
      }
      setNodes(nodes =>
        nodes.map(n => {
          if (n.id === draggedNode.id) {
            n = draggedNode;
          }
          return n;
        }),
      );
    }
  };

  return {
    handleAddToBatch,
    handleRemoveFromBatch,
  };
};
