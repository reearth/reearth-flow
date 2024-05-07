import { Fragment, useMemo } from "react";

import { ToggleArea } from "./components/ToggleArea";
import { PanelContent } from "./types";

export type VerticalPanelProps = {
  className?: string;
  isOpen: boolean;
  minHeight?: string; // tailwindcss height class
  maxHeight?: string; // tailwindcss height class
  headerContent?: PanelContent;
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
  headerContent,
  panelContents,
  onClick,
  onPanelToggle,
}) => {
  const classes = useMemo(
    () =>
      [baseClasses, isOpen ? maxHeight ?? "w-64" : minHeight ?? "w-[41px]", className].reduce(
        (acc, cur) => (cur ? `${acc} ${cur}` : acc),
      ),
    [className, isOpen, maxHeight, minHeight],
  );

  return (
    <div className={classes} onClick={() => onClick?.(isOpen)}>
      {headerContent && (
        <div className="flex flex-col gap-2 py-2 px-1">
          {isOpen ? (
            <div>{headerContent.component}</div>
          ) : headerContent.icon ? (
            <div key={headerContent.id}>{headerContent.icon}</div>
          ) : (
            <Fragment key={headerContent.id}>{headerContent.component}</Fragment>
          )}
          <div className="border-zinc-700/50 border-t-[1px] w-[100%]" />
        </div>
      )}
      <div
        className={`flex flex-1 flex-col py-2 px-1 gap-3 transition-all overflow-auto ${!isOpen ? "self-center" : "w-full"}`}>
        {panelContents?.map(content => {
          return isOpen ? (
            <div className="flex flex-col gap-2 overflow-auto" key={content.id}>
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
      <ToggleArea
        arrowDirection={isOpen ? "left" : "right"}
        arrowPosition={isOpen ? "end" : "start"}
        onClick={() => onPanelToggle?.(!isOpen)}
      />
    </div>
  );
};

export { VerticalPanel };
