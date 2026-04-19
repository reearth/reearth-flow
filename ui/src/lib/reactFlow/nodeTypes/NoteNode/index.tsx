import { NoteIcon } from "@phosphor-icons/react";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo, useMemo } from "react";

import { useAwarenessNodeSelections } from "@flow/features/Editor/editorContext";
import type { Node } from "@flow/types";

import { convertHextoRgba } from "../utils";

export type NoteNodeProps = NodeProps<Node>;

const minSize = { width: 250, height: 150 };

const NoteNode: React.FC<NoteNodeProps> = ({ id, type, data, ...props }) => {
  // background color will always be a hex color, therefore needs to be converted to rgba
  const backgroundColor = data.customizations?.backgroundColor || "";
  const rgbaColor = convertHextoRgba(backgroundColor, 0.5);
  const awarenessSelections = useAwarenessNodeSelections(id);
  const remoteColor = awarenessSelections[0]?.color;

  const gradientStyles = useMemo(() => {
    if (awarenessSelections.length < 2) return null;
    const colors = awarenessSelections.map((s) => s.color).join(", ");
    const gradient = `linear-gradient(135deg, ${colors}) border-box`;
    const fill = rgbaColor || "var(--secondary)";
    return {
      body: {
        border: "1px solid transparent",
        background: `linear-gradient(${fill}, ${fill}) padding-box, ${gradient}`,
        minWidth: minSize.width,
        minHeight: minSize.height,
      } as React.CSSProperties,
      header: {
        border: "1px solid transparent",
        background: `linear-gradient(var(--secondary), var(--secondary)) padding-box, ${gradient}`,
      } as React.CSSProperties,
    };
  }, [awarenessSelections, rgbaColor]);
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
        />
      )}
      <div
        className={`relative z-0 h-full rounded-b-lg p-1 shadow-md shadow-secondary backdrop-blur-xs ${gradientStyles ? "" : `border-x border-b bg-secondary/50 ${props.selected ? "border-border" : "border-transparent"}`}`}
        ref={(element) => {
          if (element && !gradientStyles) {
            element.style.setProperty(
              "background-color",
              rgbaColor,
              "important",
            );
          }
        }}
        style={
          gradientStyles?.body ?? {
            ...(remoteColor ? { outline: `solid ${remoteColor}` } : {}),
            minWidth: minSize.width,
            minHeight: minSize.height,
          }
        }>
        <div
          className={`absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-lg bg-secondary p-1 ${gradientStyles ? "" : `border-x border-t ${props.selected ? "border-border" : "border-transparent"}`}`}
          style={
            gradientStyles?.header ??
            (remoteColor ? { outline: `solid ${remoteColor}` } : undefined)
          }
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
                  data.customizations?.textColor || "",
                  "important",
                );
            }
          }}>
          <div
            className="nowheel nodrag size-full resize-none bg-transparent text-xs focus-visible:outline-hidden [&_a]:text-blue-400 [&_a]:underline [&_b]:font-bold [&_i]:italic [&_li]:ml-1 [&_ol]:list-decimal [&_ol]:pl-4 [&_pre]:rounded [&_pre]:bg-muted [&_pre]:p-1 [&_pre]:font-mono [&_s]:line-through [&_u]:underline [&_ul]:list-disc [&_ul]:pl-4"
            dangerouslySetInnerHTML={{
              __html: data.customizations?.content ?? "",
            }}
          />
        </div>
      </div>
    </>
  );
};

export default memo(NoteNode);
