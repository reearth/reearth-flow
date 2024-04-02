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
  panelContents?: Omit<PanelContent, "title">[];
  onToggle?: (open: boolean) => void;
};

const baseClasses = "flex flex-col box-content transition-width duration-300 ease-in-out";

const baseButtonClasses = "w-[60px] hover:bg-zinc-900";

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
    isOpen ? maxHeight ?? "h-64" : minHeight ?? "h-[36px]",
    className,
  ].reduce((acc, cur) => (cur ? `${acc} ${cur}` : acc));

  const handleToggle = useCallback(() => onToggle?.(!isOpen), [isOpen, onToggle]);

  const handleSelection = useCallback(
    (content: PanelContent) => {
      setSelected(content);
      if (!isOpen) {
        handleToggle();
      }
    },
    [isOpen, handleToggle],
  );

  useEffect(() => {
    if (!selected) {
      setSelected(panelContents?.[0]);
    }
  }, [selected, panelContents]);

  return (
    <div className={classes} onClick={handleToggle}>
      <div id="edge" className="flex gap-1 items-center h-[36px]">
        {arrowPosition === "start" && <ArrowButton direction={arrowDirection} />}
        <div className="flex gap-1 items-center justify-center flex-1 h-[100%]">
          {panelContents?.map(content => (
            <IconButton
              key={content.id}
              className={`w-[55px] h-[80%] ${selected?.id === content.id ? "text-white bg-zinc-800" : undefined}`}
              icon={content.icon}
              onClick={() => handleSelection(content)}
            />
          ))}
        </div>
      </div>
      <div id="content" className="flex flex-1 bg-zinc-800">
        {isOpen ? (
          <div className="flex flex-1 p-1" key={selected?.id}>
            {selected?.component}
          </div>
        ) : null}
      </div>
      {arrowPosition === "end" && <ArrowButton direction={arrowDirection} />}
    </div>
  );
};

export { HorizontalPanel };

const ArrowButton = ({ direction }: { direction: ArrowDirection }) => {
  return direction === "down" ? (
    <ChevronDownIcon className={baseButtonClasses} />
  ) : (
    <ChevronUpIcon className={baseButtonClasses} />
  );
};
