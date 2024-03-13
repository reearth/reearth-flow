import { MixerVerticalIcon, Pencil2Icon, ReaderIcon } from "@radix-ui/react-icons";
import { useCallback, useMemo, useState } from "react";

import { Button } from "@flow/components";

type Tool = {
  id: string;
  name: string;
  icon: React.ReactNode;
};

const Toolbox: React.FC = () => {
  const [isHovered, setIsHovered] = useState(false);

  const availableTools = useMemo<Tool[]>(
    () => [
      {
        id: "reader-node",
        name: "Reader Node",
        icon: <ReaderIcon />,
      },
      {
        id: "transformer-node",
        name: "Transformer Node",
        icon: <MixerVerticalIcon />,
      },
      {
        id: "writer-node",
        name: "Writer Node",
        icon: <Pencil2Icon />,
      },
    ],
    [],
  );

  const handleMouseOver = useCallback(() => !isHovered && setIsHovered(true), [isHovered]);
  const handleMouseLeave = useCallback(() => isHovered && setIsHovered(false), [isHovered]);

  return (
    <div
      className="flex flex-col flex-wrap bg-zinc-800 border border-zinc-600 rounded-md absolute top-3 mb-3 left-3 mr-3 text-zinc-400 transition-all"
      onMouseOver={handleMouseOver}
      onMouseLeave={handleMouseLeave}>
      {availableTools.map(tool => (
        <Button
          key={tool.id}
          className="transition-all hover:bg-zinc-600 hover:text-zinc-300"
          variant="ghost"
          size="icon">
          {tool.icon}
        </Button>
      ))}
    </div>
  );
};

export { Toolbox };
