import { RectangleDashed } from "@phosphor-icons/react";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo, useState } from "react";

import { Node } from "@flow/types";

export type BatchNodeProps = NodeProps<Node>;

export const initialSize = { width: 300, height: 200 };

export const baseBatchNode = {
  type: "batch",
  style: { width: initialSize.width + "px", height: initialSize.height + "px" },
  zIndex: -1001,
};

const minSize = { width: 250, height: 150 };

const BatchNode: React.FC<BatchNodeProps> = ({ data, selected }) => {
  const [_width, _setWidth] = useState(data.width ?? initialSize.width);
  const [_height, _setHeight] = useState(data.height ?? initialSize.height);

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
          minWidth={minSize.width}
          minHeight={minSize.height}
          onResize={(r) => {
            // setWidth(props.xPos + r.x);
            // setHeight(props.yPos + r.y);
            console.log("ADS: ", r);
          }}
        />
      )}
      <div
        className={`relative z-0 h-full rounded-b-sm bg-accent/20 ${selected ? "border-border" : undefined}`}>
        <div
          className={`absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-sm border-x border-t bg-accent/50 px-2 py-1 ${selected ? "border-border" : "border-transparent"}`}>
          <RectangleDashed />
          <p>{data.customName || data.officialName}</p>
        </div>
      </div>
    </>
  );
};

export default memo(BatchNode);
