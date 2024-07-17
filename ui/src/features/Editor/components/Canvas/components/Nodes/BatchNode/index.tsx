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
  // const BatchNode: React.FC<NodeProps<NodeData>> = ({ data, selected, ...props }) => {
  const [_width, _setWidth] = useState(data.width ?? initialSize.width);
  const [_height, _setHeight] = useState(data.height ?? initialSize.height);
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  // console.log(width, height);

  // console.log("ADS props: ", props);
  return (
    <>
      {selected && (
        // <NodeResizeControl
        //   minWidth={width < minSize.width ? minSize.width : width}
        //   minHeight={height < minSize.height ? minSize.height : height}
        //   onResize={r => {
        //     // setWidth(props.xPos + r.x);
        //     // setHeight(props.yPos + r.y);
        //     console.log("ADS: ", r);
        //   }}
        // />
        <NodeResizer
          lineStyle={{
            background: "none",
            borderColor: "transparent",
            // borderColor: "rgba(255, 255, 0, 0.8)",
            zIndex: 0,
            // borderRadius: "4px",
            // padding: 2,
          }}
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
          onResize={r => {
            // setWidth(props.xPos + r.x);
            // setHeight(props.yPos + r.y);
            console.log("ADS: ", r);
          }}
        />
      )}
      <div
        className={`relative z-0 h-full rounded-b-sm border-x border-b border-transparent bg-yellow-200/20 ${selected ? "border-yellow-200/50" : undefined}`}>
        <div
          className={`absolute -inset-x-[0.8px] -top-[33px] flex items-center gap-2 rounded-t-sm border-x border-t border-transparent bg-yellow-200/50 px-2 py-1 ${selected ? "border-yellow-200/50" : undefined}`}>
          <RectangleDashed />
          <p>{data.name}</p>
        </div>
      </div>
    </>
  );
};

export default memo(BatchNode);
