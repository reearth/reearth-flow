import { useReactFlow } from "@xyflow/react";
import { useCallback } from "react";

import { Node } from "@flow/types";

export default () => {
  const { getInternalNode, isNodeIntersecting } = useReactFlow();

  const handleAddToBatch = useCallback(
    (draggedNode: Node, hoveredNode: Node, nodes: Node[]) => {
      const internalNode = getInternalNode(draggedNode.id);
      const updatedNode: Node = { ...draggedNode };

      // Check if dragged node isn't already a child to the group
      if (!draggedNode.parentId) {
        updatedNode.parentId = hoveredNode.id;

        const posX = internalNode?.position.x || updatedNode.position.x;
        const posY = internalNode?.position.y || updatedNode.position.y;

        if (posX && posY) {
          updatedNode.position = {
            x: posX - hoveredNode.position.x,
            y: posY - hoveredNode.position.y,
          };
        }
        const newNodes: Node[] = nodes.filter((n) => n.id !== updatedNode.id);
        newNodes.push(updatedNode);
        return newNodes;
      } else {
        return nodes;
      }
    },
    [getInternalNode],
  );

  const handleRemoveFromBatch = useCallback(
    (draggedNode: Node, hoveredNode: Node, nodes: Node[]) => {
      const internalNode = getInternalNode(draggedNode.id);
      const updatedNode: Node = { ...draggedNode };

      // Check if dragged node is a child to the group
      if (draggedNode.parentId === hoveredNode.id) {
        updatedNode.parentId = undefined;

        const posX = internalNode?.position.x;
        const posY = internalNode?.position.y;
        if (posX && posY) {
          updatedNode.position = {
            x: posX + hoveredNode.position.x,
            y: posY + hoveredNode.position.y,
          };
        }

        return nodes.map((n) => {
          if (n.id === draggedNode.id) {
            n = updatedNode;
          }
          return n;
        });
      } else {
        return nodes;
      }
    },
    [getInternalNode],
  );

  const handleNodeDropInBatch = useCallback(
    (droppedNode: Node, nodes: Node[]) => {
      let newNodes: Node[] = nodes;

      nodes.forEach((nd) => {
        if (nd.type === "batch") {
          //safety check to make sure there's a height and width
          if (nd.measured?.height && nd.measured?.width) {
            const rec = {
              height: nd.measured.height,
              width: nd.measured.width,
              ...nd.position,
            };

            // Check if the dragged node is inside the group
            if (isNodeIntersecting(droppedNode, rec, false)) {
              newNodes = handleAddToBatch(droppedNode, nd, newNodes);
            } else {
              newNodes = handleRemoveFromBatch(droppedNode, nd, newNodes);
            }
          }
        }
      });
      return newNodes;
    },
    [handleAddToBatch, handleRemoveFromBatch, isNodeIntersecting],
  );

  return {
    handleNodeDropInBatch,
  };
};
