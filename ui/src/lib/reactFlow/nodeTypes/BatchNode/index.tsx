import { RectangleDashedIcon } from "@phosphor-icons/react";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo, useMemo } from "react";

import { useAwarenessNodeSelections } from "@flow/features/Editor/editorContext";
import type { Node } from "@flow/types";

import useHooks from "./hooks";

export type BatchNodeProps = NodeProps<Node> & {
  readonly?: boolean;
};

const BatchNode: React.FC<BatchNodeProps> = ({
  data,
  readonly = false,
  selected,
  id,
}) => {
  const { bounds, rgbaColor, handleOnEndResize } = useHooks({ id, data });
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
      } as React.CSSProperties,
      header: {
        border: "1px solid transparent",
        background: `linear-gradient(var(--secondary), var(--secondary)) padding-box, ${gradient}`,
      } as React.CSSProperties,
    };
  }, [awarenessSelections, rgbaColor]);

  return (
    <>
      {selected && (
        <NodeResizer
          lineStyle={{
            background: "none",
            zIndex: 0,
          }}
          shouldResize={() => !readonly}
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
        className={`relative z-0 h-full rounded-b-lg p-2 shadow-md shadow-secondary backdrop-blur-xs ${gradientStyles ? "" : `border-x border-b bg-orange-400/40 dark:bg-orange-400/20 ${selected ? "border-orange-400/50" : "border-transparent"}`} ${data.isDisabled ? "opacity-70" : ""} ${readonly ? "nopan" : ""}`}
        style={
          gradientStyles?.body ??
          (remoteColor ? { outline: `solid ${remoteColor}` } : undefined)
        }
        ref={(element) => {
          if (element && !gradientStyles) {
            element.style.setProperty(
              "background-color",
              rgbaColor,
              "important",
            );
          }
        }}>
        <div
          style={
            gradientStyles?.header ??
            (remoteColor ? { outline: `solid ${remoteColor}` } : undefined)
          }
          className={`absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-lg bg-secondary p-1 ${gradientStyles ? "" : `border-x border-t ${selected ? "border-orange-400/50" : "border-transparent"}`} ${data.isDisabled ? "opacity-70" : ""}`}
          ref={(element) => {
            if (element)
              element.style.setProperty(
                "color",
                data.customizations?.textColor || "",
                "important",
              );
          }}>
          <div className="rounded-lg bg-primary p-1">
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
