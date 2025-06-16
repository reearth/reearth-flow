import { RectangleDashedIcon } from "@phosphor-icons/react";
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
          lineClassName="border-none rounded"
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
        className={`relative z-0 h-full rounded-b-md bg-orange-400/20 p-2 border-x border-b ${selected ? "border-orange-400/50" : "border-transparent"}`}
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
          className={`absolute inset-x-[-0.8px] top-[-33px] p-1 flex items-center gap-2 rounded-t-md border-x border-t bg-secondary px-2 ${selected ? "border-orange-400/50" : "border-transparent"}`}
          ref={(element) => {
            if (element)
              element.style.setProperty(
                "color",
                data.customizations?.textColor || "",
                "important",
              );
          }}>
          <div className="p-1 rounded-sm bg-primary">
            <RectangleDashedIcon
              className="w-[15px] fill-orange-400/80"
              weight="bold"
            />
          </div>
          <p>{data.customizations?.customName || data.officialName}</p>
        </div>
      </div>
    </>
  );
};

export default memo(BatchNode);
