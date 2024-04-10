import { useState } from "react";
import { NodeProps, NodeResizer } from "reactflow";

type NodeData = {
  content: string;
  width?: number;
  height?: number;
  backgroundColor?: string;
  textColor?: string;
};

export const initialSize = { width: 300, height: 200 };

// export const baseBatchNode = {
//   type: "batch",
//   style: { width: initialSize.width + "px", height: initialSize.height + "px" },
//   zIndex: -1001,
// };

const minSize = { width: 250, height: 150 };

const NoteNode: React.FC<NodeProps<NodeData>> = ({ data, ...props }) => {
  const [width, _setWidth] = useState(data.width ?? initialSize.width);
  const [height, _setHeight] = useState(data.height ?? initialSize.height);
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  console.log(width, height);

  console.log("ADS props: ", props);
  return (
    <>
      {props.selected && (
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
            borderColor: "rgba(0, 0, 255, 0.8)",
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
      {/* <div className={`bg-orange-400/60 w-[${width}px] h-[${height}px]`} style={{ width, height }}> */}
      <div className={`bg-blue-200/20 rounded-sm h-full z-0 p-2`}>
        <textarea
          className="resize-none w-full h-auto bg-transparent nowheel"
          defaultValue={data.content}
          onMouseDown={e => e.stopPropagation()}
          // onMouseUp={e => e.}
        />
      </div>
    </>
  );
};

export default NoteNode;
