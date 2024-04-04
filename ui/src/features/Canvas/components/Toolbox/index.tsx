import { useCallback, useMemo, useState } from "react";

import { IconButton, ReaderIcon, TransformerIcon, WriterIcon } from "@flow/components";
import { useT } from "@flow/providers";

type Tool = {
  id: string;
  name: string;
  icon: React.ReactNode;
};

type Props = {
  className?: string;
};

const Toolbox: React.FC<Props> = ({ className }) => {
  const t = useT();
  const [isHovered, setIsHovered] = useState(false);

  const availableTools = useMemo<Tool[]>(
    () => [
      {
        id: "reader-node",
        name: t("Reader Node"),
        icon: <ReaderIcon />,
      },
      {
        id: "transformer-node",
        name: t("Transformer Node"),
        icon: <TransformerIcon />,
      },
      {
        id: "writer-node",
        name: t("Writer Node"),
        icon: <WriterIcon />,
      },
    ],
    [t],
  );

  const handleMouseOver = useCallback(() => !isHovered && setIsHovered(true), [isHovered]);
  const handleMouseLeave = useCallback(() => isHovered && setIsHovered(false), [isHovered]);

  return (
    <div
      className={`flex flex-col flex-wrap bg-zinc-800 border border-zinc-600 rounded-md text-zinc-400 transition-all ${className}`}
      onMouseOver={handleMouseOver}
      onMouseLeave={handleMouseLeave}>
      {availableTools.map(tool => (
        <IconButton
          key={tool.id}
          tooltipPosition="right"
          tooltipText={tool.name}
          icon={tool.icon}
        />
      ))}
    </div>
  );
};

export { Toolbox };
