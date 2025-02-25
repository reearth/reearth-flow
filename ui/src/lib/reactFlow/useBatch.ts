import { useReactFlow } from "@xyflow/react";
import { useCallback } from "react";

import { Node } from "@flow/types";

export default () => {
  const { getInternalNode, getIntersectingNodes } = useReactFlow();

  const calculateNewRelativePosition = useCallback(
    (droppedNode: Node, parentNode: Node) => {
      // If the dragged node has a parent already, and is moving to a new parent,
      // we need to un-offset it first, before re-offsetting it to the new parent.
      if (droppedNode.parentId) {
        const oldParentNode = getInternalNode(droppedNode.parentId);
        if (oldParentNode) {
          const actualCanvasPosition = {
            x: droppedNode.position.x + oldParentNode.position.x,
            y: droppedNode.position.y + oldParentNode.position.y,
          };
          return {
            x: actualCanvasPosition.x - parentNode.position.x,
            y: actualCanvasPosition.y - parentNode.position.y,
          };
        }
      }

      return {
        x: droppedNode.position.x - parentNode.position.x,
        y: droppedNode.position.y - parentNode.position.y,
      };
    },
    [getInternalNode],
  );

  const handleAddToBatch = useCallback(
    (droppedNode: Node, parentNode: Node) => {
      const updatedNode: Node = { ...droppedNode };

      // Check if dragged node isn't already a child to the group
      if (droppedNode.parentId !== parentNode.id) {
        updatedNode.parentId = parentNode.id;

        const updatedPosition = calculateNewRelativePosition(
          droppedNode,
          parentNode,
        );
        if (updatedPosition) {
          updatedNode.position = updatedPosition;
        }

        return updatedNode;
      } else {
        return droppedNode;
      }
    },
    [calculateNewRelativePosition],
  );

  const handleRemoveFromBatch = useCallback(
    (draggedNode: Node): Node => {
      if (!draggedNode.parentId) return draggedNode;
      const internalNode = getInternalNode(draggedNode.id);
      const updatedNode: Node = { ...draggedNode };

      const parentNode = getInternalNode(draggedNode.parentId);

      // Check if dragged node is a child to the group

      updatedNode.parentId = undefined;

      // This return will cause the dragged node's position to be moved unexpectedly.
      // In most cases, this return will not happen, so its okay (though not ideal).
      if (!parentNode) return updatedNode;

      const posX = internalNode?.position.x;
      const posY = internalNode?.position.y;
      if (posX && posY) {
        updatedNode.position = {
          x: posX + parentNode.position.x,
          y: posY + parentNode.position.y,
        };
      }

      return updatedNode;
    },
    [getInternalNode],
  );

  const handleNodesDropInBatch = useCallback(
    (droppedNodes: Node[]): Node[] | undefined => {
      if (droppedNodes.length === 0) return;

      let newNodes: Node[] | undefined = undefined;

      const intersectingSets = droppedNodes.map(
        (n) => new Set(getIntersectingNodes(n, false) as Node[]),
      );

      const parentNode = [...intersectingSets[0]]
        .filter((node) => intersectingSets.every((set) => set.has(node)))
        .find((node) => node.type === "batch");

      droppedNodes.forEach((node) => {
        //safety check to make sure there's a height and width
        if (node.measured?.height && node.measured?.width) {
          // Check if the dragged node is inside the batch
          if (parentNode) {
            newNodes = [
              ...(newNodes ?? []),
              handleAddToBatch(node, parentNode),
            ];
            // Check if the dragged node is a child to a batch
          } else if (node.parentId) {
            newNodes = [...(newNodes ?? []), handleRemoveFromBatch(node)];
          }
        }
      });

      return newNodes;
    },
    [getIntersectingNodes, handleAddToBatch, handleRemoveFromBatch],
  );

  return {
    handleNodesDropInBatch,
  };
};
