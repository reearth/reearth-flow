import { useReactFlow } from "@xyflow/react";
import { useCallback } from "react";

import {
  DEFAULT_BATCH_MIN_SIZE,
  DEFAULT_BATCH_PADDING,
} from "@flow/global-constants";
import { Node, NodeData } from "@flow/types";

import useBatch from "../../useBatch";
import { convertHextoRgba } from "../utils";

export default ({ id, data }: { id: string; data: NodeData }) => {
  const { getNodes, updateNode } = useReactFlow<Node>();
  const { handleNodesDropInBatch } = useBatch();

  const getChildNodesBoundary = useCallback(() => {
    const nodes = getNodes();
    const childNodes = nodes.filter((node) => node.parentId === id);

    let maxX = 0;
    let maxY = 0;

    childNodes.forEach((node) => {
      if (node.measured) {
        const rightEdge = node.position.x + (node.measured?.width ?? 0);
        const bottomEdge = node.position.y + (node.measured?.height ?? 0);

        maxX = Math.max(maxX, rightEdge);
        maxY = Math.max(maxY, bottomEdge);
      }
    });

    return {
      // Add padding to the maxX and maxY to show that node cannot be resized beyond the placement of child nodes
      width: Math.max(
        DEFAULT_BATCH_MIN_SIZE.width,
        maxX + DEFAULT_BATCH_PADDING,
      ),
      height: Math.max(
        DEFAULT_BATCH_MIN_SIZE.height,
        maxY + DEFAULT_BATCH_PADDING,
      ),
    };
  }, [getNodes, id]);

  const handleOnEndResize = useCallback(() => {
    const allNodes = getNodes();
    const initialParentCount = allNodes.filter((node) => node.parentId).length;

    const batchableNodes = allNodes.filter(
      (node) => node.type !== "batch" && !node.parentId,
    );

    const updatedNodes = handleNodesDropInBatch(batchableNodes);
    const finalParentCount = updatedNodes?.filter(
      (node) => node.parentId,
    ).length;

    if (finalParentCount !== initialParentCount) {
      updatedNodes?.forEach((node) => {
        updateNode(node.id, node, { replace: true });
      });
    }
  }, [getNodes, updateNode, handleNodesDropInBatch]);

  // No need to memoize as we want to update because bounds will change on resize
  const bounds = getChildNodesBoundary();
  // background color will always be a hex color, therefore needs to be converted to rgba
  const backgroundColor = data.customizations?.backgroundColor || "";
  const rgbaColor = convertHextoRgba(backgroundColor, 0.5);

  return {
    bounds,
    rgbaColor,
    handleOnEndResize,
  };
};
