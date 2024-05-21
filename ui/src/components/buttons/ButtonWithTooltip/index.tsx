import { Tooltip, TooltipContent, TooltipTrigger } from "@flow/components";

import { Button, type ButtonProps } from "../BaseButton";

export type ButtonWithTooltipProps = {
  tooltipText: string;
  tooltipPosition?: "top" | "right" | "bottom" | "left";
  tooltipOffset?: number;
} & ButtonProps;

const ButtonWithTooltip: React.FC<ButtonWithTooltipProps> = ({
  tooltipText,
  tooltipPosition = "bottom",
  tooltipOffset,
  children,
  ...props
}) => (
  <Tooltip>
    <TooltipTrigger asChild>
      <Button {...props}>{children}</Button>
    </TooltipTrigger>
    <TooltipContent side={tooltipPosition} sideOffset={tooltipOffset}>
      <p>{tooltipText}</p>
    </TooltipContent>
  </Tooltip>
);

export { ButtonWithTooltip };
