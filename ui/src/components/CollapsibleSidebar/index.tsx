import { ToggleArea } from "./ToggleArea";

export type Props = {
  className?: string;
  isOpen: boolean;
  direction?: "horizontal" | "vertical";
  minHeight?: string; // tailwindcss height class
  maxHeight?: string; // tailwindcss height class
  togglePosition?: "start" | "end";
  toggleSidebar?: () => void;
  children?: React.ReactNode;
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
  children,
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
  ].reduce((acc, cur) => {
    if (cur) {
      return `${acc} ${cur}`;
    }
    return acc;
  });

  return (
    <div className={classes}>
      {togglePosition === "start" && (
        <ToggleArea
          arrowDirection={arrowDirection}
          arrowPosition={arrowPosition}
          onClick={toggleSidebar}
        />
      )}
      <div className="flex-1 overflow-hidden">{children}</div>
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
