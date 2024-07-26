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
      icon: <ArrowArcLeft className="size-4 stroke-1" weight="thin" />,
    },
    {
      id: "redo",
      name: t("Redo action"),
      icon: <ArrowArcRight className="size-4 stroke-1" weight="thin" />,
    },
  ];

  const onDragStart = (event: DragEvent<HTMLButtonElement>, nodeType: NodeType) => {
    event.dataTransfer.setData("application/reactflow", nodeType);
    event.dataTransfer.effectAllowed = "move";
  };

  return (
    <div className="pointer-events-none absolute bottom-1 left-2 top-2 flex shrink-0 gap-2 [&>*]:pointer-events-auto">
      <div className="self-start rounded-md bg-secondary">
        <div className="flex flex-col flex-wrap rounded-md border transition-all">
          {availableTools.map(tool => (
            <IconButton
              key={tool.id}
              className={`dndnode-${tool.id} rounded-[4px]`}
              tooltipPosition="right"
              tooltipText={tool.name}
              icon={tool.icon}
              onDragStart={event => onDragStart(event, tool.id)}
              draggable
            />
          ))}
          {availableActions && <div className="my-2 w-full border-t" />}
          {availableActions.map(action => (
            <IconButton
              key={action.id}
              className="rounded-[4px]"
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

export default memo(Toolbox);
