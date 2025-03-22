import {
  Database,
  Disc,
  Eye,
  GearFine,
  Graph,
  Lightning,
  Table,
  Trash,
  X,
} from "@phosphor-icons/react";
import { NodeProps } from "@xyflow/react";
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

import { Handles } from "./components";
import useHooks from "./hooks";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
};

const typeIconClasses = "w-[10px] h-[100%]";

const GeneralNode: React.FC<GeneralNodeProps> = ({
  className,
  id,
  data,
  type,
  selected,
}) => {
  const t = useT();

  const { onNodesChange, onSecondaryNodeAction } = useEditorContext();

  const handleNodeDelete = useCallback(() => {
    onNodesChange?.([{ id, type: "remove" }]);
  }, [id, onNodesChange]);

  const handleSecondaryNodeAction = useCallback(() => {
    if (!id) return;
    onSecondaryNodeAction?.(undefined, id, data.subworkflowId);
  }, [id, data.subworkflowId, onSecondaryNodeAction]);

  const {
    officialName,
    customName,
    inputs,
    outputs,
    status,
    intermediateDataUrl,
    borderColor,
    selectedColor,
    selectedBackgroundColor,
  } = useHooks({ data, type });

  return (
    <ContextMenu>
      <ContextMenuTrigger>
        <div
          className={`rounded-sm bg-secondary ${status === "running" ? "active-node-status-shadow" : status === "pending" ? "queued-node-status-shadow" : ""}`}>
          <div className="relative z-[1001] flex h-[25px] w-[150px] rounded-sm">
            <div
              className={`flex w-4 justify-center rounded-l-sm border-y border-l ${selected ? selectedColor : borderColor} ${selected ? selectedBackgroundColor : className}`}>
              {type === "reader" ? (
                <Database className={typeIconClasses} />
              ) : type === "writer" ? (
                <Disc className={typeIconClasses} />
              ) : type === "transformer" ? (
                <Lightning className={typeIconClasses} />
              ) : type === "subworkflow" ? (
                <Graph className={typeIconClasses} />
              ) : null}
            </div>
            <div
              className={`flex flex-1 items-center justify-between gap-2 truncate rounded-r-sm border-y border-r px-1 leading-none ${selected ? selectedColor : borderColor}`}>
              <p className="self-center truncate text-xs dark:font-light">
                {data.customizations?.customName || customName || officialName}
              </p>
              {status === "failed" && <X className="size-4 text-destructive" />}
              {status === "succeeded" && intermediateDataUrl && (
                <Table className="size-4 text-success" />
              )}
            </div>
          </div>
          <Handles nodeType={type} inputs={inputs} outputs={outputs} />
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
  );
};

export default memo(GeneralNode);
