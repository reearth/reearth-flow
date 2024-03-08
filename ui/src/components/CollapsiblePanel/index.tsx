import { Fragment } from "react";

import { ToggleArea } from "./ToggleArea";

export type Props = {
  className?: string;
  isOpen: boolean;
  direction?: "horizontal" | "vertical";
  minHeight?: string; // tailwindcss height class
  maxHeight?: string; // tailwindcss height class
  togglePosition?: "start" | "end";
  panelContents?: PanelContent[];
  onPanelToggle?: (open: boolean) => void;
};

export type PanelContent = {
  id: string;
  component: React.ReactNode;
  title?: string;
  icon?: React.ReactNode;
};

const baseClasses = `flex box-content transition-width duration-300 ease-in-out`;

const CollapsiblePanel: React.FC<Props> = ({
  className,
  isOpen,
  direction = "vertical",
  minHeight,
  maxHeight,
  togglePosition = "start",
  panelContents,
  onPanelToggle,
}) => {
  const arrowPosition = isOpen ? "end" : "center";

  const arrowDirection =
    direction === "horizontal" ? (isOpen ? "down" : "up") : isOpen ? "left" : "right";

  const classes = [
    baseClasses,
    direction === "vertical" ? "flex-col" : undefined,
    direction === "horizontal"
      ? isOpen
        ? maxHeight ?? "h-64"
        : minHeight ?? "h-[36px]"
      : isOpen
        ? maxHeight ?? "w-64"
        : minHeight ?? "w-[41px]",
    className,
  ].reduce((acc, cur) => (cur ? `${acc} ${cur}` : acc));

  return (
    <div className={classes}>
      {togglePosition === "start" && (
        <ToggleArea
          arrowDirection={arrowDirection}
          arrowPosition={arrowPosition}
          onClick={() => onPanelToggle?.(!isOpen)}
        />
      )}
      <div
        className={`flex flex-1 ${direction === "horizontal" ? "px-3" : "flex-col py-3"} gap-3 overflow-hidden transition-all ${!isOpen ? "self-center" : "w-[250px]"}`}>
        {panelContents?.map(content => {
          return isOpen ? (
            <div
              className={`flex ${direction === "vertical" ? "flex-col" : undefined} gap-2`}
              key={content.id}>
              {content.title && <p className="text-lg">{content.title}</p>}
              {content.component}
            </div>
          ) : content.icon ? (
            <div key={content.id}>{content.icon}</div>
          ) : (
            <Fragment key={content.id}>{content.component}</Fragment>
          );
        })}
      </div>
      {togglePosition === "end" && (
        <ToggleArea
          arrowDirection={arrowDirection}
          arrowPosition={arrowPosition}
          onClick={() => onPanelToggle?.(!isOpen)}
        />
      )}
    </div>
  );
};

export { CollapsiblePanel };
