import { Fragment, useMemo } from "react";

import { ToggleArea } from "./components/ToggleArea";
import { PanelContent } from "./types";

export type VerticalPanelProps = {
  className?: string;
  isOpen: boolean;
  minHeight?: string; // tailwindcss height class
  maxHeight?: string; // tailwindcss height class
  togglePosition?: "start-left" | "start-right" | "end-left" | "end-right";
  panelContents?: PanelContent[];
  onPanelToggle?: (open: boolean) => void;
  onClick?: (currentOpenState?: boolean) => void; // optional onClick handler
};

const baseClasses = `flex flex-col box-content transition-width duration-300 ease-in-out`;

const VerticalPanel: React.FC<VerticalPanelProps> = ({
  className,
  isOpen,
  minHeight,
  maxHeight,
  togglePosition = "start-right",
  panelContents,
  onClick,
  onPanelToggle,
}) => {
  const arrowPosition = useMemo(
    () => (isOpen ? (togglePosition.includes("left") ? "start" : "end") : "center"),
    [isOpen, togglePosition],
  );

  const arrowDirection = useMemo(
    () =>
      togglePosition.includes("left") ? (isOpen ? "right" : "left") : isOpen ? "left" : "right",
    [isOpen, togglePosition],
  );

  const classes = useMemo(
    () =>
      [baseClasses, isOpen ? maxHeight ?? "w-64" : minHeight ?? "w-[41px]", className].reduce(
        (acc, cur) => (cur ? `${acc} ${cur}` : acc),
      ),
    [className, isOpen, maxHeight, minHeight],
  );

  return (
    <div className={classes} onClick={() => onClick?.(isOpen)}>
      {togglePosition.includes("start") && (
        <ToggleArea
          arrowDirection={arrowDirection}
          arrowPosition={arrowPosition}
          onClick={() => onPanelToggle?.(!isOpen)}
        />
      )}
      <div
        className={`flex flex-1 flex-col py-2 px-1 gap-3 overflow-scroll transition-all ${!isOpen ? "self-center" : "w-[250px]"}`}>
        {panelContents?.map(content => {
          return isOpen ? (
            <div className="flex flex-col gap-2" key={content.id}>
              {content.title && <p className="text-md">{content.title}</p>}
              {content.component}
            </div>
          ) : content.icon ? (
            <div key={content.id}>{content.icon}</div>
          ) : (
            <Fragment key={content.id}>{content.component}</Fragment>
          );
        })}
      </div>
      {togglePosition.includes("end") && (
        <ToggleArea
          arrowDirection={arrowDirection}
          arrowPosition={arrowPosition}
          onClick={() => onPanelToggle?.(!isOpen)}
        />
      )}
    </div>
  );
};

export { VerticalPanel };
