import { NoteIcon } from "@phosphor-icons/react";
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
        className={`relative z-0 h-full rounded-b-lg border-x border-b bg-secondary/50 p-1 shadow-md shadow-secondary backdrop-blur-xs ${props.selected ? "border-border" : "border-transparent"}`}
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
          className={`absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-lg border-x border-t bg-secondary p-1 ${props.selected ? "border-border" : "border-transparent"}`}
          ref={(element) => {
            if (element)
              element.style.setProperty(
                "color",
                data.customizations?.textColor || "",
                "important",
              );
          }}>
          <div className="rounded-lg bg-primary/80 p-1">
            <NoteIcon className="w-[15px]" />
          </div>
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
