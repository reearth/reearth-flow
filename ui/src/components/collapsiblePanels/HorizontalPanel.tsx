import { useEffect, useMemo, useState } from "react";

import { IconButton } from "..";

import { ToggleArea } from "./components/ToggleArea";
import { PanelContent } from "./types";

export type HorizontalPanelProps = {
  className?: string;
  isOpen: boolean;
  minHeight?: string; // tailwindcss height class
  maxHeight?: string; // tailwindcss height class
  togglePosition?: "start-left" | "start-right" | "end-left" | "end-right";
  panelContents?: Omit<PanelContent, "title">[];
  onPanelToggle?: (open: boolean) => void;
  onClick?: (currentOpenState?: boolean) => void; // optional onClick handler
};

const baseClasses = "flex flex-col box-content transition-width duration-300 ease-in-out";

const baseButtonClasses = "w-[60px]";

const HorizontalPanel: React.FC<HorizontalPanelProps> = ({
  className,
  isOpen,
  minHeight,
  maxHeight,
  togglePosition = "start-right",
  panelContents,
  onClick,
  onPanelToggle,
}) => {
  const [selected, setSelected] = useState(panelContents?.[0]);

  const arrowPosition = useMemo(
    () => (isOpen ? (togglePosition.includes("left") ? "start" : "end") : "center"),
    [isOpen, togglePosition],
  );

  const arrowDirection = useMemo(
    () => (togglePosition.includes("right") ? (isOpen ? "down" : "up") : isOpen ? "up" : "down"),
    [isOpen, togglePosition],
  );

  const classes = useMemo(
    () =>
      [baseClasses, isOpen ? maxHeight ?? "h-64" : minHeight ?? "h-[36px]", className].reduce(
        (acc, cur) => (cur ? `${acc} ${cur}` : acc),
      ),
    [className, isOpen, maxHeight, minHeight],
  );

  useEffect(() => {
    if (!selected) {
      setSelected(panelContents?.[0]);
    }
  }, [selected, panelContents]);

  return (
    <div className={classes} onClick={() => onClick?.(isOpen)}>
      <div id="edge" className="flex gap-1 items-center">
        {togglePosition.includes("start") && (
          <ToggleArea
            buttonClassName={baseButtonClasses}
            arrowDirection={arrowDirection}
            arrowPosition={arrowPosition}
            onClick={() => onPanelToggle?.(!isOpen)}
          />
        )}
        {panelContents?.map(content => (
          <IconButton
            key={content.id}
            className={`w-[55px] h-[80%] ${selected?.id === content.id ? "text-white bg-zinc-800" : undefined}`}
            icon={content.icon}
            onClick={() => setSelected(content)}
          />
        ))}
      </div>
      <div id="content" className="flex flex-1 bg-zinc-800">
        {isOpen ? (
          <div className="flex flex-1 p-2" key={selected?.id}>
            {selected?.component}
          </div>
        ) : null}
      </div>
      {togglePosition.includes("end") && (
        <ToggleArea
          buttonClassName={baseButtonClasses}
          arrowDirection={arrowDirection}
          arrowPosition={arrowPosition}
          onClick={() => onPanelToggle?.(!isOpen)}
        />
      )}
    </div>
  );
};

export { HorizontalPanel };
