import { RectangleDashed } from "@phosphor-icons/react";
import { NodeProps, NodeResizer, useReactFlow } from "@xyflow/react";
import { memo, useState, useCallback } from "react";

import { Node } from "@flow/types";

import useBatch from "../../../useBatch";

export type BatchNodeProps = NodeProps<Node>;

export const initialSize = { width: 300, height: 200 };

export const baseBatchNode = {
  type: "batch",
  style: { width: initialSize.width + "px", height: initialSize.height + "px" },
  zIndex: -1001,
};

const minSize = { width: 250, height: 150 };

const BatchNode: React.FC<BatchNodeProps> = ({ data, selected, id }) => {
  const [_width, _setWidth] = useState(data.width ?? initialSize.width);
  const [_height, _setHeight] = useState(data.height ?? initialSize.height);
  const { getNodes, setNodes } = useReactFlow<Node>();
  const { handleNodeDropInBatch } = useBatch();

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
      // Add 8px padding to the maxX and maxY to show that node cannot be resized beyond the placement of child nodes
      width: Math.max(minSize.width, maxX + 8),
      height: Math.max(minSize.height, maxY + 8),
    };
  }, [getNodes, id]);

  const handleOnEndResize = useCallback(() => {
    const allNodes = getNodes();
    let updatedNodes = allNodes;
    const initialParentCount = allNodes.filter((node) => node.parentId).length;

    const nonBatchNodes = allNodes.filter((node) => node.type !== "batch");
    nonBatchNodes.forEach((node) => {
      updatedNodes = handleNodeDropInBatch(node, updatedNodes);
    });
    const finalParentCount = updatedNodes.filter(
      (node) => node.parentId,
    ).length;

    if (finalParentCount !== initialParentCount) {
      setNodes(updatedNodes);
    }
  }, [getNodes, setNodes, handleNodeDropInBatch]);
  // No need to memoize as we want to update because bounds will change on resize
  const bounds = getChildNodesBoundary();

  const params = data.params as unknown as BatchNodeParams | undefined;

  return (
    <>
      {selected && (
        <NodeResizer
          lineStyle={{
            background: "none",
            zIndex: 0,
          }}
          lineClassName="border border-border rounded"
          handleStyle={{
            background: "none",
            width: 8,
            height: 8,
            border: "none",
            borderRadius: "80%",
            zIndex: 0,
          }}
          minWidth={bounds.width}
          minHeight={bounds.height}
          onResizeEnd={handleOnEndResize}
        />
      )}
      <div
        className={`bg-accent/20 relative z-0 h-full rounded-b-sm ${selected ? "border-border" : undefined}`}>
        <div
          className={`bg-accent/50 absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-sm border-x border-t px-2 py-1 ${selected ? "border-border" : "border-transparent"}`}>
          <RectangleDashed />
          <p>{data.customName || data.officialName}</p>
        </div>
      </div>
    </>
  );
};

export default memo(BatchNode);
