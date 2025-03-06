import { RectangleDashed } from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo } from "react";

import { Node, NodeType } from "@flow/types";

import NodeContextMenu from "../NodeContextMenu";

import useHooks from "./hooks";

export type BatchNodeProps = NodeProps<Node>;

export const initialSize = { width: 300, height: 200 };

const batchNodeCustomizationSchema: RJSFSchema = {
  type: "object",
  properties: {
    customName: { type: "string", title: "Name" },
    backgroundColor: {
      type: "string",
      format: "color",
      default: "#323236",
      title: "Background Color",
    },
    textColor: {
      type: "string",
      format: "color",
      title: "Text Color",
      default: "#fafafa",
    },
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
  customization: batchNodeCustomizationSchema,
};

export const baseBatchNode: {
  type: NodeType;
  style: { width: string; height: string };
  zIndex: number;
} = {
  type: "batch",
  style: { width: initialSize.width + "px", height: initialSize.height + "px" },
  zIndex: -1001,
};

const BatchNode: React.FC<BatchNodeProps> = ({ data, selected, type, id }) => {
  const { bounds, rgbaColor, handleOnEndResize } = useHooks({ id, data });

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

      <NodeContextMenu nodeId={id} nodeType={type}>
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
                  data.customizations?.textColor || "",
                  "important",
                );
            }}>
            <RectangleDashed />
            <p>{data.customizations?.customName || data.officialName}</p>
          </div>
        </div>
      </NodeContextMenu>
    </>
  );
};

export default memo(BatchNode);
