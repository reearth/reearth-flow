import { RectangleDashed } from "@phosphor-icons/react";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo } from "react";

import type { Node } from "@flow/types";

import useHooks from "./hooks";

export type BatchNodeProps = NodeProps<Node>;

const BatchNode: React.FC<BatchNodeProps> = ({ data, selected, id }) => {
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
                data.customizations?.textColor || "",
                "important",
              );
          }}>
          <RectangleDashed />
          <p>{data.customizations?.customName || data.officialName}</p>
        </div>
      </div>
    </>
  );
};

export default memo(BatchNode);
