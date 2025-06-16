import {
  ArrowArcLeft,
  ArrowArcRight,
  Database,
  Disc,
  Graph,
  Layout,
  Lightning,
  Note,
  RectangleDashed,
} from "@phosphor-icons/react";
import { memo, type DragEvent } from "react";
import { createRoot } from "react-dom/client";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { NodeType } from "@flow/types";

type BreakItem = { id: "break" };

type ToolboxItem<T> = {
  id: T;
  name: string;
  icon: React.ReactNode;
  disabled?: boolean;
  onClick?: () => void;
};

type Tool = ToolboxItem<NodeType> | BreakItem;

type CanvasAction = "layout" | "undo" | "redo";
type Action = ToolboxItem<CanvasAction> | BreakItem;

type Props = {
  canUndo: boolean;
  canRedo: boolean;
  isMainWorkflow: boolean;
  hasReader?: boolean;
  onRedo: () => void;
  onUndo: () => void;
  onLayoutChange: () => void;
};

const Toolbox: React.FC<Props> = ({
  canUndo,
  canRedo,
  isMainWorkflow,
  hasReader,
  onRedo,
  onUndo,
  onLayoutChange,
}) => {
  const t = useT();
  const availableTools: Tool[] = [
    {
      id: "reader" as const,
      name: t("Reader Node"),
      icon: <Database weight="thin" size={16} />,
      disabled: !isMainWorkflow || hasReader,
    },
    {
      id: "transformer" as const,
      name: t("Transformer Node"),
      icon: <Lightning weight="thin" size={16} />,
    },
    {
      id: "writer" as const,
      name: t("Writer Node"),
      icon: <Disc weight="thin" size={16} />,
      disabled: !isMainWorkflow,
    },
    {
      id: "note" as const,
      name: t("Note"),
      icon: <Note weight="thin" size={16} />,
    },
    {
      id: "batch" as const,
      name: t("Batch Node"),
      icon: (
        <RectangleDashed className="fill-orange-400" weight="thin" size={16} />
      ),
    },
    {
      id: "subworkflow" as const,
      name: t("Subworkflow Node"),
      icon: <Graph weight="thin" size={16} />,
    },
  ];

  const availableActions: Action[] = [
    {
      id: "layout",
      name: t("Auto layout"),
      icon: <Layout weight="thin" size={16} />,
      onClick: onLayoutChange,
    },
    { id: "break" },
    {
      id: "undo",
      name: t("Undo last action"),
      icon: <ArrowArcLeft weight="thin" size={16} />,
      disabled: !canUndo,
      onClick: onUndo,
    },
    {
      id: "redo",
      name: t("Redo action"),
      icon: <ArrowArcRight weight="thin" size={16} />,
      disabled: !canRedo,
      onClick: onRedo,
    },
  ];

  const onDragStart = (
    event: DragEvent<HTMLButtonElement>,
    nodeType: NodeType,
  ) => {
    event.dataTransfer.setData("application/reactflow", nodeType);
    event.dataTransfer.effectAllowed = "move";
    const dragPreviewContainer = document.createElement("div");
    dragPreviewContainer.style.position = "absolute";
    dragPreviewContainer.style.top = "-1000px"; // Move it offscreen to hide it

    const root = createRoot(dragPreviewContainer);
    root.render(
      <div className="rounded bg-secondary">
        <div
          className={`
          flex size-9 justify-center rounded align-middle
          ${
            nodeType === "reader"
              ? "bg-node-reader/60"
              : nodeType === "writer"
                ? "bg-node-writer/60"
                : nodeType === "subworkflow"
                  ? "bg-node-entrance"
                  : nodeType === "note" || nodeType === "batch"
                    ? "bg-primary"
                    : "bg-node-transformer/60"
          }`}>
          {nodeType === "reader" ? (
            <Database className="self-center" />
          ) : nodeType === "writer" ? (
            <Disc className="self-center" />
          ) : nodeType === "subworkflow" ? (
            <Graph className="self-center" />
          ) : nodeType === "batch" ? (
            <RectangleDashed className="self-center" />
          ) : nodeType === "note" ? (
            <Note className="self-center" />
          ) : (
            <Lightning className="self-center" />
          )}
        </div>
      </div>,
    );

    document.body.appendChild(dragPreviewContainer);
    event.dataTransfer.setDragImage(dragPreviewContainer, 10, 10);

    // Clean up the container after the drag starts
    setTimeout(() => {
      root.unmount();
      document.body.removeChild(dragPreviewContainer);
    }, 0);
  };

  return (
    <div className="self-start rounded-md bg-secondary/80 p-1 shadow-md backdrop-blur-xs">
      <div className="flex flex-col flex-wrap gap-1 rounded-md transition-all">
        {availableTools.map((tool, idx) =>
          tool.id === "break" ? (
            <div key={tool.id + idx} className="mx-1 box-border border-t" />
          ) : (
            <div key={tool.id} className="rounded-md bg-secondary">
              <IconButton
                className={`dndnode-${tool.id} cursor-grab  ${
                  tool.id === "reader"
                    ? "bg-node-reader/50 hover:bg-node-reader/80"
                    : tool.id === "writer"
                      ? "bg-node-writer/50 hover:bg-node-writer/80"
                      : tool.id === "subworkflow"
                        ? "bg-node-subworkflow/50 hover:bg-node-subworkflow/80"
                        : tool.id === "batch" || tool.id === "note"
                          ? "bg-primary/50 hover:bg-primary/80"
                          : "bg-node-transformer/50 hover:bg-node-transformer/80"
                }`}
                tooltipPosition="right"
                tooltipOffset={4}
                showArrow
                tooltipText={tool.name}
                icon={tool.icon}
                onDragStart={(event) => onDragStart(event, tool.id)}
                draggable
                disabled={tool.disabled}
              />
            </div>
          ),
        )}
        <div className="mx-1 box-border border-t" />
        {availableActions.map((action, idx) =>
          action.id === "break" ? (
            <div key={action.id + idx} className="mx-1 box-border border-t" />
          ) : (
            <IconButton
              key={action.id}
              className="gap-0 rounded-[4px] hover:bg-primary/60"
              tooltipPosition="right"
              tooltipText={action.name}
              tooltipOffset={4}
              showArrow
              icon={action.icon}
              disabled={action.disabled}
              onClick={action.onClick}
            />
          ),
        )}
      </div>
    </div>
  );
};

export default memo(Toolbox);
