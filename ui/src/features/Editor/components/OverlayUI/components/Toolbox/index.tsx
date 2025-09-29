import {
  ArrowArcLeftIcon,
  ArrowArcRightIcon,
  DatabaseIcon,
  DiscIcon,
  GraphIcon,
  LayoutIcon,
  LightningIcon,
  NoteIcon,
  RectangleDashedIcon,
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
  onRedo: () => void;
  onUndo: () => void;
  onLayoutChange: () => void;
};

const Toolbox: React.FC<Props> = ({
  canUndo,
  canRedo,
  isMainWorkflow,
  onRedo,
  onUndo,
  onLayoutChange,
}) => {
  const t = useT();
  const availableTools: Tool[] = [
    {
      id: "reader" as const,
      name: t("Reader Node"),
      icon: <DatabaseIcon weight="thin" size={16} />,
    },
    {
      id: "transformer" as const,
      name: t("Transformer Node"),
      icon: <LightningIcon weight="thin" size={16} />,
    },
    {
      id: "writer" as const,
      name: t("Writer Node"),
      icon: <DiscIcon weight="thin" size={16} />,
      disabled: !isMainWorkflow,
    },
    {
      id: "note" as const,
      name: t("Note"),
      icon: <NoteIcon weight="thin" size={16} />,
    },
    {
      id: "batch" as const,
      name: t("Batch Node"),
      icon: (
        <RectangleDashedIcon
          className="fill-orange-400"
          weight="thin"
          size={16}
        />
      ),
    },
    {
      id: "subworkflow" as const,
      name: t("Subworkflow Node"),
      icon: <GraphIcon weight="thin" size={16} />,
    },
  ];

  const availableActions: Action[] = [
    {
      id: "layout",
      name: t("Auto layout"),
      icon: <LayoutIcon weight="thin" size={16} />,
      onClick: onLayoutChange,
    },
    { id: "break" },
    {
      id: "undo",
      name: t("Undo last action"),
      icon: <ArrowArcLeftIcon weight="thin" size={16} />,
      disabled: !canUndo,
      onClick: onUndo,
    },
    {
      id: "redo",
      name: t("Redo action"),
      icon: <ArrowArcRightIcon weight="thin" size={16} />,
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
          flex h-8 w-18 justify-center rounded align-middle
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
            <DatabaseIcon className="self-center" />
          ) : nodeType === "writer" ? (
            <DiscIcon className="self-center" />
          ) : nodeType === "subworkflow" ? (
            <GraphIcon className="self-center" />
          ) : nodeType === "batch" ? (
            <RectangleDashedIcon className="self-center" />
          ) : nodeType === "note" ? (
            <NoteIcon className="self-center" />
          ) : (
            <LightningIcon className="self-center" />
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
    <div className="self-start rounded-xl border border-primary bg-secondary/70 p-1 shadow-md shadow-secondary backdrop-blur-xs">
      <div className="flex flex-wrap gap-2 rounded-md transition-all">
        {availableTools.map((tool, idx) =>
          tool.id === "break" ? (
            <div key={tool.id + idx} className="mx-1 box-border border-t" />
          ) : (
            <div key={tool.id} className="self-center rounded-md bg-secondary">
              <IconButton
                className={`dndnode-${tool.id} h-8 w-18 cursor-grab backdrop-blur-xs ${
                  tool.id === "reader"
                    ? "bg-node-reader/40 hover:bg-node-reader/80"
                    : tool.id === "writer"
                      ? "bg-node-writer/40 hover:bg-node-writer/80"
                      : tool.id === "subworkflow"
                        ? "bg-node-subworkflow/40 hover:bg-node-subworkflow/80"
                        : tool.id === "batch" || tool.id === "note"
                          ? "bg-primary/40 hover:bg-primary/80"
                          : "bg-node-transformer/40 hover:bg-node-transformer/80"
                }`}
                tooltipPosition="bottom"
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
        <div className="my-1 border-r" />
        {availableActions.map((action, idx) =>
          action.id === "break" ? (
            <div key={action.id + idx} className="my-1 border-r" />
          ) : (
            <IconButton
              key={action.id}
              className="h-8 w-10 gap-0 rounded-[4px] hover:bg-primary/60"
              tooltipPosition="bottom"
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
