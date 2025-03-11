import {
  Eye,
  GearFine,
  Graph,
  RectangleDashed,
  Trash,
} from "@phosphor-icons/react";
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

import useHooks from "./hooks";

export type BatchNodeProps = NodeProps<Node>;

const BatchNode: React.FC<BatchNodeProps> = ({ data, selected, type, id }) => {
  const t = useT();

  const { onNodesChange, onSecondaryNodeAction } = useEditorContext();

  const handleNodeDelete = useCallback(() => {
    onNodesChange?.([{ id, type: "remove" }]);
  }, [id, onNodesChange]);

  const handleSecondaryNodeAction = useCallback(() => {
    if (!id) return;
    onSecondaryNodeAction?.(undefined, id, data.subworkflowId);
  }, [id, data.subworkflowId, onSecondaryNodeAction]);

  const { bounds, rgbaColor, handleOnEndResize } = useHooks({ id, data });

  return (
    <>
      {selected && (
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
          minWidth={bounds.width}
          minHeight={bounds.height}
          onResize={() => "asldfkjsadf"}
          onResizeEnd={handleOnEndResize}
        />
      )}

      <ContextMenu>
        <ContextMenuTrigger>
          <div
            className={`relative z-0 h-full rounded-b-sm bg-accent/20 ${selected ? "border-border" : undefined}`}
            ref={(element) => {
              if (element) {
                element.style.setProperty(
                  "background-color",
                  rgbaColor,
                  "important",
                );
              }
            }}>
            <div
              className={`absolute inset-x-[-0.8px] top-[-33px] flex items-center gap-2 rounded-t-sm border-x border-t bg-accent/50 px-2 py-1 ${selected ? "border-border" : "border-transparent"}`}
              ref={(element) => {
                if (element)
                  element.style.setProperty(
                    "color",
                    data.customizations?.textColor || "",
                    "important",
                  );
              }}>
              <RectangleDashed />
              <p>{data.customizations?.customName || data.officialName}</p>
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

export default memo(BatchNode);
