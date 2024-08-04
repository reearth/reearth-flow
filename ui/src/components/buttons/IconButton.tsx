import { Button } from "./BaseButton";
import {
  ButtonWithTooltip,
  type ButtonWithTooltipProps,
} from "./ButtonWithTooltip";

type Props = Omit<ButtonWithTooltipProps, "tooltipText"> & {
  icon: React.ReactNode;
  tooltipText?: string;
};

const IconButton: React.FC<Props> = ({
  className,
  icon,
  tooltipText,
  ...props
}) => {
  return tooltipText ? (
    <ButtonWithTooltip
      className={`cursor-pointer transition-all ${className}`}
      variant="ghost"
      size="icon"
      tooltipText={tooltipText}
      {...props}
    >
      {icon}
    </ButtonWithTooltip>
  ) : (
    <Button
      className={`cursor-pointer transition-all ${className}`}
      variant="ghost"
      size="icon"
      {...props}
    >
      {icon}
    </Button>
  );
};

export { IconButton };
