import {
  ArrowArcLeft,
  ArrowArcRight,
  Database,
  Disc,
  Graph,
  Lightning,
  Note,
  RectangleDashed,
} from "@phosphor-icons/react";
import { memo, type DragEvent } from "react";
import { createRoot } from "react-dom/client";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { NodeType } from "@flow/types";

type ToolboxItem<T> = {
  id: T;
  name: string;
  icon: React.ReactNode;
};

type Tool = ToolboxItem<NodeType>;

type CanvasAction = "undo" | "redo";
type Action = ToolboxItem<CanvasAction>;

type Props = {
  canUndo: boolean;
  canRedo: boolean;
  onRedo?: () => void;
  onUndo?: () => void;
  isMainWorkflow: boolean;
  hasReader: boolean;
};

const Toolbox: React.FC<Props> = ({
  canUndo,
  canRedo,
  onRedo,
  onUndo,
  isMainWorkflow,
  hasReader,
}) => {
  const t = useT();
  const availableTools: Tool[] = [
    {
      id: "reader" as const,
      name: t("Reader Node"),
      icon: <Database weight="thin" />,
    },
    {
      id: "transformer" as const,
      name: t("Transformer Node"),
      icon: <Lightning weight="thin" />,
    },
    {
      id: "writer" as const,
      name: t("Writer Node"),
      icon: <Disc weight="thin" />,
    },
    {
      id: "note" as const,
      name: t("Note"),
      icon: <Note weight="thin" />,
    },
    {
      id: "batch" as const,
      name: t("Batch Node"),
      icon: <RectangleDashed weight="thin" />,
    },
    {
      id: "subworkflow" as const,
      name: t("Subworkflow Node"),
      icon: <Graph weight="thin" />,
    },
  ].filter((tool) => {
    if (!isMainWorkflow) {
      return tool.id !== "reader" && tool.id !== "writer";
    }

    if (isMainWorkflow && hasReader) {
      return tool.id !== "reader";
    }
    return true;
  });

  const availableActions: Action[] = [
    {
      id: "undo",
      name: t("Undo last action"),
      icon: <ArrowArcLeft className="size-4 stroke-1" weight="thin" />,
    },
    {
      id: "redo",
      name: t("Redo action"),
      icon: <ArrowArcRight className="size-4 stroke-1" weight="thin" />,
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
      <div className="flex size-12 rounded bg-secondary">
        <div
          className={`
          flex w-full justify-center rounded align-middle
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
    <div className="pointer-events-none absolute bottom-1 left-2 top-2 flex shrink-0 gap-2 [&>*]:pointer-events-auto">
      <div className="self-start rounded-md bg-secondary">
        <div className="flex flex-col flex-wrap rounded-md border transition-all">
          {availableTools.map((tool) => (
            <IconButton
              key={tool.id}
              className={`dndnode-${tool.id} rounded-[4px]`}
              tooltipPosition="right"
              tooltipText={tool.name}
              icon={tool.icon}
              onDragStart={(event) => onDragStart(event, tool.id)}
              draggable
            />
          ))}
          {availableActions && <div className="my-2 w-full border-t" />}
          {availableActions.map((action) => (
            <IconButton
              key={action.id}
              className="rounded-[4px]"
              tooltipPosition="right"
              tooltipText={action.name}
              icon={action.icon}
              disabled={action.id === "undo" ? !canUndo : !canRedo}
              onClick={() =>
                action.id === "redo"
                  ? onRedo?.()
                  : action.id === "undo"
                    ? onUndo?.()
                    : undefined
              }
            />
          ))}
        </div>
      </div>
    </div>
  );
};

export default memo(Toolbox);
