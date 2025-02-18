import { Note } from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo } from "react";

import { Node } from "@flow/types";

import { convertHextoRgba } from "../utils";

export type NoteNodeProps = NodeProps<Node>;

export const initialSize = { width: 300, height: 200 };
const minSize = { width: 250, height: 150 };

// TODO: Currently textarea data.content on node is not setting the value correctly. Temporary fix is to use text on RJSFS params @billcookie
const noteNodeSchema: RJSFSchema = {
  type: "object",
  properties: {
    customName: { type: "string", title: "Name" },
    backgroundColor: {
      type: "string",
      format: "color",
      title: "Background Color",
    },
    text: { type: "string", format: "textarea", title: "Text" },
    textColor: { type: "string", format: "color", title: "Text Color" },
  },
};

export const noteNodeAction = {
  name: "note",
  description: "Note node",
  type: "note",
  categories: ["note"],
  inputPorts: ["input"],
  outputPorts: ["output"],
  builtin: true,
  parameter: noteNodeSchema,
};

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
  // background color will always be a hex color, therefore needs to be converted to rgba
  const backgroundColor = data.params?.backgroundColor || "";
  const rgbaColor = convertHextoRgba(backgroundColor, 0.5);

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
        ref={(element) => {
          if (element) {
            element.style.setProperty(
              "background-color",
              rgbaColor,
              "important",
            );
          }
        }}
        style={{
          minWidth: minSize.width,
          minHeight: minSize.height,
        }}>
        <div
          className={`absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-sm border-x border-t bg-accent/50 px-2 py-1 ${props.selected ? "border-border" : "border-transparent"}`}
          ref={(element) => {
            if (element)
              element.style.setProperty(
                "color",
                data.params?.textColor || "",
                "important",
              );
          }}>
          <Note />
          <p>{data.params?.customName ?? data.officialName}</p>
        </div>
        <span
          style={{
            minWidth: "inherit",
            minHeight: "inherit",
          }}
          ref={(element) => {
            if (element)
              element.style.setProperty(
                "color",
                data.params?.textColor || "",
                "important",
              );
          }}
          className="nowheel nodrag size-full resize-none bg-transparent text-xs focus-visible:outline-none">
          {data.params?.text}
        </span>
        {/* <textarea
          defaultValue={data.content}
          style={{
            minWidth: "inherit",
            minHeight: "inherit",
          }}
          ref={(element) => {
            if (element)
              element.style.setProperty(
                "color",
                data.params?.textColor || "",
                "important",
              );
          }}
          className="nowheel nodrag size-full resize-none bg-transparent text-xs focus-visible:outline-none"
        /> */}
      </div>
    </>
  );
};

export default memo(NoteNode);
