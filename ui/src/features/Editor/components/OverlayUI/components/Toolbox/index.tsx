import {
  ArrowArcLeft,
  ArrowArcRight,
  Database,
  Disc,
  Lightning,
  Note,
  RectangleDashed,
} from "@phosphor-icons/react";
import { type DragEvent } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { type NodeType } from "@flow/types";

type ToolboxItem<T> = {
  id: T;
  name: string;
  icon: React.ReactNode;
};

type Tool = ToolboxItem<NodeType>;

type CanvasAction = "undo" | "redo";
type Action = ToolboxItem<CanvasAction>;

type Props = {
  undoDisabled?: boolean;
  onRedo?: () => void;
  onUndo?: () => void;
};

const Toolbox: React.FC<Props> = ({ onRedo, onUndo }) => {
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
  ];

  const availableActions: Action[] = [
    {
      id: "undo",
      name: t("Undo last action"),
      icon: <ArrowArcLeft className="h-4 w-4 stroke-1" weight="thin" />,
    },
    {
      id: "redo",
      name: t("Redo action"),
      icon: <ArrowArcRight className="h-4 w-4 stroke-1" weight="thin" />,
    },
  ];

  const onDragStart = (event: DragEvent<HTMLButtonElement>, nodeType: NodeType) => {
    event.dataTransfer.setData("application/reactflow", nodeType);
    event.dataTransfer.effectAllowed = "move";
  };

  return (
    <div className="absolute left-2 top-2 bottom-1 flex flex-shrink-0 gap-2 pointer-events-none [&>*]:pointer-events-auto">
      <div className="bg-zinc-800 self-start">
        <div className="flex flex-col flex-wrap bg-zinc-900/50 border border-zinc-700 rounded-md text-zinc-400 transition-all">
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
    </div>
  );
};

export { Toolbox };
