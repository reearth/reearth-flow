import { Note } from "@phosphor-icons/react";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo } from "react";

import { Node } from "@flow/types";

export type NoteNodeProps = NodeProps<Node>;

export const initialSize = { width: 300, height: 200 };
const minSize = { width: 250, height: 150 };

export const baseNoteNode = {
  type: "note",
  content: "New Note",
  measured: {
    width: initialSize.width,
    height: initialSize.height,
  },
  style: {
    width: `${initialSize.width}px`,
    height: `${initialSize.height}px`,
    minWidth: `${minSize.width}px`,
    minHeight: `${minSize.height}px`,
  },
};

const NoteNode: React.FC<NoteNodeProps> = ({ data, ...props }) => {
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
          // onResize={(r) => {
          //   console.log("ADS: ", r);
          // }}
        />
      )}
      <div
        className="z-0 h-full rounded-sm bg-secondary/50 p-2"
        style={{
          minWidth: minSize.width,
          minHeight: minSize.height,
        }}>
        <div
          className={`absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-sm border-x border-t bg-accent/50 px-2 py-1 ${props.selected ? "border-border" : "border-transparent"}`}>
          <Note />
          <p>{data.customName ?? data.officialName}</p>
        </div>
        <textarea
          defaultValue={data.content}
          style={{
            minWidth: "inherit",
            minHeight: "inherit",
          }}
          className="nowheel nodrag size-full resize-none bg-transparent text-xs focus-visible:outline-none"
        />
      </div>
    </>
  );
};

export default memo(NoteNode);
