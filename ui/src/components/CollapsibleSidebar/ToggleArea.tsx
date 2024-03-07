import {
  ChevronDownIcon,
  ChevronUpIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from "@radix-ui/react-icons";

import { Button } from "..";

export type ArrowPosition = "center" | "end";
export type ArrowDirection = "left" | "right" | "up" | "down";

export type ToggleProps = {
  className?: string;
  buttonClassName?: string;
  arrowPosition?: ArrowPosition;
  arrowDirection?: ArrowDirection;
  onClick?: () => void;
};

const ToggleArea: React.FC<ToggleProps> = ({
  className,
  buttonClassName,
  arrowPosition,
  arrowDirection,
  onClick,
}) => (
  <div
    className={`flex ${arrowPosition === "end" ? "justify-end" : "justify-center"} w-fill cursor-pointer ${className}`}
    onClick={onClick}>
    <Button
      className={`hover:bg-zinc-800 hover:text-white ${buttonClassName}`}
      variant="ghost"
      size="icon">
      {arrowDirection === "right" ? (
        <ChevronRightIcon />
      ) : arrowDirection === "up" ? (
        <ChevronUpIcon />
      ) : arrowDirection === "down" ? (
        <ChevronDownIcon />
      ) : (
        <ChevronLeftIcon />
      )}
    </Button>
  </div>
);

export { ToggleArea };
