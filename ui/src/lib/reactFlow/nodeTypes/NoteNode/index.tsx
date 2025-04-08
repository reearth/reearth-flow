import { Note } from "@phosphor-icons/react";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo } from "react";

import type { Node } from "@flow/types";

import { convertHextoRgba } from "../utils";

export type NoteNodeProps = NodeProps<Node>;

const minSize = { width: 250, height: 150 };

const NoteNode: React.FC<NoteNodeProps> = ({ id, type, data, ...props }) => {
  // background color will always be a hex color, therefore needs to be converted to rgba
  const backgroundColor = data.customizations?.backgroundColor || "";
  const rgbaColor = convertHextoRgba(backgroundColor, 0.5);

  return (
    <>
      {props.selected && (
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
          minWidth={minSize.width}
          minHeight={minSize.height}
          // onResize={(r) => {
          //   console.log("ADS: ", r);
          // }}
        />
      )}
      <div
        className={`relative z-0 h-full rounded-b-md bg-secondary/50 p-2 border ${props.selected ? "border-border" : "border-transparent"}`}
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
          className={`absolute inset-x-[-0.8px] top-[-26px] flex items-center gap-2 rounded-t-md border-x border-t bg-accent/50 px-2 ${props.selected ? "border-border" : "border-transparent"}`}
          ref={(element) => {
            if (element)
              element.style.setProperty(
                "color",
                data.customizations?.textColor || "",
                "important",
              );
          }}>
          <Note />
          <p>{data.customizations?.customName ?? data.officialName}</p>
        </div>
        <div
          className=""
          ref={(element) => {
            if (element) {
              if (element)
                element.style.setProperty(
                  "color",
                  data.params?.textColor || "",
                  "important",
                );
            }
          }}>
          <p className="nowheel nodrag size-full resize-none bg-transparent text-xs focus-visible:outline-hidden">
            {data.customizations?.content}
          </p>
        </div>
      </div>
    </>
  );
};

export default memo(NoteNode);
