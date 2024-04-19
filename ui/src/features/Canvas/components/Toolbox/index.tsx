import { DiscIcon, GroupIcon, Pencil2Icon } from "@radix-ui/react-icons";
import { type DragEvent } from "react";

import { IconButton, ReaderIcon, TransformerIcon } from "@flow/components";
import { useT } from "@flow/providers";

import { type NodeType } from "../Nodes/GeneralNode/types";

type Tool = {
  id: NodeType;
  name: string;
  icon: React.ReactNode;
};

type Props = {
  className?: string;
};

const Toolbox: React.FC<Props> = ({ className }) => {
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

  const onDragStart = (event: DragEvent<HTMLButtonElement>, nodeType: NodeType) => {
    event.dataTransfer.setData("application/reactflow", nodeType);
    event.dataTransfer.effectAllowed = "move";
  };

  return (
    <div
      className={`flex flex-col flex-wrap bg-zinc-800 border border-zinc-600/60 rounded-md text-zinc-400 transition-all ${className}`}>
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
    </div>
  );
};

export { Toolbox };
