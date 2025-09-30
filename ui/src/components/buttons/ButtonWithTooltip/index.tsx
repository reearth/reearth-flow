import { Tooltip, TooltipContent, TooltipTrigger } from "@flow/components";

import { Button, type ButtonProps } from "../BaseButton";

export type ButtonWithTooltipProps = {
  tooltipText: string;
  tooltipPosition?: "top" | "right" | "bottom" | "left";
  tooltipOffset?: number;
  showArrow?: boolean;
  delayDuration?: number;
} & ButtonProps;

const ButtonWithTooltip: React.FC<ButtonWithTooltipProps> = ({
  tooltipText,
  tooltipPosition = "bottom",
  tooltipOffset,
  children,
  showArrow,
  delayDuration = 700,
  ...props
}) => (
  <Tooltip delayDuration={delayDuration}>
    <TooltipTrigger asChild>
      <Button {...props}>{children}</Button>
    </TooltipTrigger>
    <TooltipContent
      side={tooltipPosition}
      sideOffset={tooltipOffset}
      showArrow={showArrow}>
      <p>{tooltipText}</p>
    </TooltipContent>
  </Tooltip>
);

export { ButtonWithTooltip };
