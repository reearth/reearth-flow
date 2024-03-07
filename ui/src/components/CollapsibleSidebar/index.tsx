import { Fragment } from "react";

import { ToggleArea } from "./ToggleArea";

export type Props = {
  className?: string;
  isOpen: boolean;
  direction?: "horizontal" | "vertical";
  minHeight?: string; // tailwindcss height class
  maxHeight?: string; // tailwindcss height class
  togglePosition?: "start" | "end";
  toggleSidebar?: () => void;
  sidebarContents?: SidebarContent[];
};

export type SidebarContent = {
  id: string;
  component: React.ReactNode;
  title?: string;
  icon?: React.ReactNode;
};

const baseClasses = `text-white z-20 flex p-[10px] box-content transition-width duration-300 ease-in-out`;

const CollapsibleSidebar: React.FC<Props> = ({
  className,
  isOpen,
  direction = "vertical",
  minHeight,
  maxHeight,
  togglePosition = "start",
  toggleSidebar,
  sidebarContents,
}) => {
  const arrowPosition = isOpen ? "end" : "center";

  const arrowDirection =
    direction === "horizontal" ? (isOpen ? "down" : "up") : isOpen ? "left" : "right";

  const classes = [
    baseClasses,
    direction === "vertical" ? "flex-col" : undefined,
    direction === "horizontal"
      ? isOpen
        ? maxHeight ?? "h-[250px]"
        : minHeight ?? "h-[50px]"
      : isOpen
        ? maxHeight ?? "w-[250px]"
        : minHeight ?? "w-[50px]",
    className,
  ].reduce((acc, cur) => (cur ? `${acc} ${cur}` : acc));

  return (
    <div className={classes}>
      {togglePosition === "start" && (
        <ToggleArea
          buttonClassName={!isOpen ? minHeight : undefined}
          arrowDirection={arrowDirection}
          arrowPosition={arrowPosition}
          onClick={toggleSidebar}
        />
      )}
      <div
        className={`flex-1 overflow-hidden w-[250px] transition-all ${!isOpen ? "w-[15px] self-center" : undefined}`}>
        {sidebarContents?.map(content => {
          return isOpen ? (
            <div className="flex flex-col gap-2" key={content.id}>
              {content.title && <p className="text-lg">{content.title}</p>}
              {content.component}
            </div>
          ) : content.icon ? (
            <div className="w-[13]" key={content.id}>
              {content.icon}
            </div>
          ) : (
            <Fragment key={content.id}>{content.component}</Fragment>
          );
        })}
      </div>
      {togglePosition === "end" && (
        <ToggleArea
          arrowDirection={arrowDirection}
          arrowPosition={arrowPosition}
          onClick={toggleSidebar}
        />
      )}
    </div>
  );
};

export { CollapsibleSidebar };
