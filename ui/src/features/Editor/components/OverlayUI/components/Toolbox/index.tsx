import {
  ArrowArcLeft,
  ArrowArcRight,
  Database,
  Disc,
  Lightning,
  Note,
  RectangleDashed,
} from "@phosphor-icons/react";
import { memo, type DragEvent } from "react";
import { createRoot } from "react-dom/client";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { type NodeType } from "@flow/types";

import { Plus } from "@phosphor-icons/react";

type ToolboxItem<T> = {
  id: T;
  name: string;
  icon: React.ReactNode;
};

type Tool = ToolboxItem<NodeType>;

type CanvasAction = "undo" | "redo" | "add";
type Action = ToolboxItem<CanvasAction>;

type Props = {
  undoDisabled?: boolean;
  onRedo?: () => void;
  onUndo?: () => void;
  onWorkflowAdd?: (position?: { x: number; y: number }) => void;
};
const Toolbox: React.FC<Props> = ({ onRedo, onUndo, onWorkflowAdd }) => {
  const t = useT();

  const availableTools: Tool[] = [
    {
      id: "reader",
      name: t("Reader Node"),
      icon: <Database weight="thin" />,
    },
    {
      id: "transformer",
      name: t("Transformer Node"),
      icon: <Lightning weight="thin" />,
    },
    {
      id: "writer",
      name: t("Writer Node"),
      icon: <Disc weight="thin" />,
    },
    {
      id: "note",
      name: t("Note"),
      icon: <Note weight="thin" />,
    },
    {
      id: "batch",
      name: t("Batch Node"),
      icon: <RectangleDashed weight="thin" />,
    },
    {
      id: "subworkflow",
      name: t("Create new sub workflow"),
      icon: <Plus className="size-4 stroke-1" weight="thin" />,
    },
  ];

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
          className={`flex w-full justify-center rounded align-middle  ${nodeType === "reader" ? "bg-node-reader/60" : nodeType === "writer" ? "bg-node-writer/60" : nodeType === "subworkflow" ? "bg-node-entrance/60" : "bg-node-transformer/60"}`}>
          <Lightning className="self-center" />
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
          <div className="my-2 w-full border-t" />
          {availableTools.map((tool) => (
            <IconButton
              key={tool.id}
              className={`dndnode-${tool.id} rounded-[4px]`}
              tooltipPosition="right"
              tooltipText={tool.name}
              icon={tool.icon}
              onDragStart={(event) => onDragStart(event, tool.id)}
              onDragEnd={(event) =>
                tool.id === "subworkflow"
                  ? onWorkflowAdd?.({ x: event.clientX - 50, y: event.clientY })
                  : undefined
              }
              onClick={() =>
                tool.id === "subworkflow" ? onWorkflowAdd?.() : undefined
              }
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
