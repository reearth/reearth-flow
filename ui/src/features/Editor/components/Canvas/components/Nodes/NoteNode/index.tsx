import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo, useState } from "react";

import { Node } from "@flow/types";

export type NoteNodeProps = NodeProps<Node>;

export const initialSize = { width: 300 };

export const baseNoteNode = {
  type: "note",
  content: "New Note",
  width: 300,
  height: 200,
};

const minSize = { width: 250, height: 150 };

const NoteNode: React.FC<NoteNodeProps> = ({ data, ...props }) => {
  const [_width, _setWidth] = useState(data.width ?? initialSize.width);
  const [_height, _setHeight] = useState(data.height);

  return (
    <>
      {props.selected && (
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
      <div className={`z-0 h-full rounded-sm bg-secondary/50 p-2`}>
        <textarea
          className="nowheel size-full resize-none bg-transparent focus-visible:outline-none"
          defaultValue={data.content}
          onMouseDown={(e) => e.stopPropagation()}
        />
      </div>
    </>
  );
};

export default memo(NoteNode);
