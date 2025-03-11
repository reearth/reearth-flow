import { Eye, GearFine, Graph, Note, Trash } from "@phosphor-icons/react";
import { NodeProps, NodeResizer } from "@xyflow/react";
import { memo, useCallback } from "react";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@flow/components";
import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import { isActionNodeType, Node } from "@flow/types";

import { convertHextoRgba } from "../utils";

export type NoteNodeProps = NodeProps<Node>;

const minSize = { width: 250, height: 150 };

const NoteNode: React.FC<NoteNodeProps> = ({ id, type, data, ...props }) => {
  const t = useT();

  const { onNodesChange, onSecondaryNodeAction } = useEditorContext();

  const handleNodeDelete = useCallback(() => {
    onNodesChange?.([{ id, type: "remove" }]);
  }, [id, onNodesChange]);

  const handleSecondaryNodeAction = useCallback(() => {
    if (!id) return;
    onSecondaryNodeAction?.(undefined, id, data.subworkflowId);
  }, [id, data.subworkflowId, onSecondaryNodeAction]);

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
      <ContextMenu>
        <ContextMenuTrigger>
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
                    data.customizations?.textColor || "",
                    "important",
                  );
              }}>
              <Note />
              <p>{data.customizations?.customName ?? data.officialName}</p>
            </div>
            <div
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
              <p className="nowheel nodrag size-full resize-none bg-transparent text-xs focus-visible:outline-none">
                {data.customizations?.content}
              </p>
            </div>
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent>
          {type === "subworkflow" ? (
            <ContextMenuItem
              className="justify-between gap-4 text-xs"
              onClick={handleSecondaryNodeAction}>
              {t("Open Subworkflow Canvas")}
              <Graph weight="light" />
            </ContextMenuItem>
          ) : (
            <ContextMenuItem
              className="justify-between gap-4 text-xs"
              onClick={handleSecondaryNodeAction}>
              {t("Node Settings")}
              <GearFine weight="light" />
            </ContextMenuItem>
          )}
          {isActionNodeType(type) && (
            <ContextMenuItem className="justify-between gap-4 text-xs" disabled>
              {t("Preview Intermediate Data")}
              <Eye weight="light" />
            </ContextMenuItem>
          )}

          {/* <ContextMenuItem
      className="justify-between gap-4 text-xs"
      disabled={!selected}>
      {t("Subworkflow from Selection")}
      <Graph weight="light" />
    </ContextMenuItem> */}
          <ContextMenuSeparator />
          <ContextMenuItem
            className="justify-between gap-4 text-xs text-destructive"
            onClick={handleNodeDelete}>
            {t("Delete Node")}
            <Trash weight="light" />
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </>
  );
};

export default memo(NoteNode);
