import { DiscIcon, GroupIcon, Pencil2Icon } from "@radix-ui/react-icons";
import { Redo, Undo } from "lucide-react";
import { type DragEvent } from "react";

import { IconButton, ReaderIcon, TransformerIcon } from "@flow/components";
import { useT } from "@flow/providers";

import { type NodeType } from "../Nodes/GeneralNode/types";

type ToolboxItem<T> = {
  id: T;
  name: string;
  icon: React.ReactNode;
};

type Tool = ToolboxItem<NodeType>;

type CanvasAction = "undo" | "redo";
type Action = ToolboxItem<CanvasAction>;

type Props = {
  className?: string;
  undoDisabled?: boolean;
  onRedo?: () => void;
  onUndo?: () => void;
};

const Toolbox: React.FC<Props> = ({ className, onRedo, onUndo }) => {
  const t = useT();

  const availableTools: Tool[] = [
    {
      id: "reader",
      name: t("Reader Node"),
      icon: <ReaderIcon />,
    },
    {
      id: "transformer",
      name: t("Transformer Node"),
      icon: <TransformerIcon />,
    },
    {
      id: "writer",
      name: t("Writer Node"),
      icon: <DiscIcon />,
    },
    {
      id: "note",
      name: t("Note"),
      icon: <Pencil2Icon />,
    },
    {
      id: "batch",
      name: t("Batch Node"),
      icon: <GroupIcon />,
    },
  ];

  const availableActions: Action[] = [
    {
      id: "undo",
      name: t("Undo last action"),
      icon: <Undo className="h-4 w-4 stroke-1" />,
    },
    {
      id: "redo",
      name: t("Redo action"),
      icon: <Redo className="h-4 w-4 stroke-1" />,
    },
  ];

  const onDragStart = (event: DragEvent<HTMLButtonElement>, nodeType: NodeType) => {
    event.dataTransfer.setData("application/reactflow", nodeType);
    event.dataTransfer.effectAllowed = "move";
  };

  return (
    <div className={`bg-zinc-800 ${className}`}>
      <div className="flex flex-col flex-wrap bg-zinc-700/40 border border-zinc-700 rounded-md text-zinc-400 transition-all">
        {availableTools.map(tool => (
          <IconButton
            key={tool.id}
            className={`dndnode-${tool.id}`}
            tooltipPosition="right"
            tooltipText={tool.name}
            icon={tool.icon}
            onDragStart={event => onDragStart(event, tool.id)}
            draggable
          />
        ))}
        {availableActions && <div className="w-full border-t border-zinc-700 my-2" />}
        {availableActions.map(action => (
          <IconButton
            key={action.id}
            tooltipPosition="right"
            tooltipText={action.name}
            icon={action.icon}
            onClick={() =>
              action.id === "redo" ? onRedo?.() : action.id === "undo" ? onUndo?.() : undefined
            }
          />
        ))}
      </div>
    </div>
  );
};

export { Toolbox };
