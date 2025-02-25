import { RectangleDashed } from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { NodeProps, NodeResizer, useReactFlow } from "@xyflow/react";
import { memo, useCallback } from "react";

import { useBatch } from "@flow/lib/reactFlow";
import { Node } from "@flow/types";

import { convertHextoRgba } from "../utils";

export type BatchNodeProps = NodeProps<Node>;

export const initialSize = { width: 300, height: 200 };

const batchNodeSchema: RJSFSchema = {
  type: "object",
  properties: {
    customName: { type: "string", title: "Name" },
    backgroundColor: {
      type: "string",
      format: "color",
      title: "Background Color",
    },
    textColor: { type: "string", format: "color", title: "Text Color" },
  },
};

export const batchNodeAction = {
  name: "batch",
  description: "Batch node",
  type: "batch",
  categories: ["batch"],
  inputPorts: ["input"],
  outputPorts: ["output"],
  builtin: true,
  parameter: batchNodeSchema,
};

export const baseBatchNode = {
  type: "batch",
  style: { width: initialSize.width + "px", height: initialSize.height + "px" },
  zIndex: -1001,
};

const minSize = { width: 250, height: 150 };

const BatchNode: React.FC<BatchNodeProps> = ({ data, selected, id }) => {
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
      // Add 8px padding to the maxX and maxY to show that node cannot be resized beyond the placement of child nodes
      width: Math.max(minSize.width, maxX + 8),
      height: Math.max(minSize.height, maxY + 8),
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
  const backgroundColor = data.params?.backgroundColor || "";
  const rgbaColor = convertHextoRgba(backgroundColor, 0.5);
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
          onResize={() => "asldfkjsadf"}
          onResizeEnd={handleOnEndResize}
        />
      )}

      <div
        className={`relative z-0 h-full rounded-b-sm bg-accent/20 ${selected ? "border-border" : undefined}`}
        ref={(element) => {
          if (element) {
            element.style.setProperty(
              "background-color",
              rgbaColor,
              "important",
            );
          }
        }}>
        <div
          className={`absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-sm border-x border-t bg-accent/50 px-2 py-1 ${selected ? "border-border" : "border-transparent"}`}
          ref={(element) => {
            if (element)
              element.style.setProperty(
                "color",
                data.params?.textColor || "",
                "important",
              );
          }}>
          <RectangleDashed />
          <p>{data.params?.customName || data.officialName}</p>
        </div>
      </div>
    </>
  );
};

export default memo(BatchNode);
