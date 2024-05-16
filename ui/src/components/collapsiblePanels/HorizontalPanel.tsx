import { ChevronDownIcon, ChevronUpIcon } from "@radix-ui/react-icons";
import { useCallback, useEffect, useState } from "react";

import { IconButton } from "..";

import { PanelContent } from "./types";

type ArrowDirection = "up" | "down";

export type HorizontalPanelProps = {
  className?: string;
  isOpen: boolean;
  minHeight?: string; // tailwindcss height class
  maxHeight?: string; // tailwindcss height class
  togglePosition?: "top-left" | "top-right" | "bottom-left" | "bottom-right";
  panelContents?: PanelContent[];
  onToggle?: (open: boolean) => void;
};

const baseClasses = "flex flex-col box-content transition-width duration-300 ease-in-out";

const HorizontalPanel: React.FC<HorizontalPanelProps> = ({
  className,
  isOpen,
  minHeight,
  maxHeight,
  togglePosition = "top-right",
  panelContents,
  onToggle,
}) => {
  const [selected, setSelected] = useState(panelContents?.[0]);

  const arrowPosition = togglePosition.includes("bottom") ? "end" : "start";

  const arrowDirection: ArrowDirection = togglePosition.includes("right")
    ? isOpen
      ? "down"
      : "up"
    : isOpen
      ? "up"
      : "down";

  const classes = [
    baseClasses,
    isOpen ? maxHeight ?? "h-100" : minHeight ?? "h-[36px]",
    className,
  ].reduce((acc, cur) => (cur ? `${acc} ${cur}` : acc));

  const handleSelection = useCallback(
    (content: PanelContent) => {
      if (content.id !== selected?.id) {
        setSelected(content);
        if (!isOpen) {
          onToggle?.(true);
        }
      } else {
        onToggle?.(!isOpen);
      }
    },
    [isOpen, onToggle, selected],
  );

  useEffect(() => {
    if (!selected) {
      setSelected(panelContents?.[0]);
    }
  }, [selected, panelContents]);

  return (
    <div className={classes}>
      <div className="flex gap-1 items-center justify-center h-[36px]">
        {panelContents?.map(content => (
          <IconButton
            key={content.id}
            className={`w-[55px] h-[80%] ${selected?.id === content.id ? "text-white bg-zinc-800" : undefined}`}
            icon={content.icon}
            tooltipText={content.description}
            tooltipPosition="top"
            onClick={() => handleSelection(content)}
          />
        ))}
      </div>
      <div id="content" className="flex flex-1 bg-zinc-800">
        {isOpen && (
          <div className="flex flex-1 p-1" key={selected?.id}>
            {selected?.component}
          </div>
        )}
      </div>
      {arrowPosition === "end" && <ArrowButton direction={arrowDirection} />}
    </div>
  );
};

export { HorizontalPanel };

const ArrowButton = ({ direction }: { direction: ArrowDirection }) => {
  return direction === "down" ? (
    <ChevronDownIcon className="w-[60px]" />
  ) : (
    <ChevronUpIcon className="w-[60px]" />
  );
};
