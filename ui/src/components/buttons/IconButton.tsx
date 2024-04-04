import { Button } from "./BaseButton";
import { ButtonWithTooltip, type ButtonWithTooltipProps } from "./ButtonWithTooltip";

type Props = Omit<ButtonWithTooltipProps, "tooltipText"> & {
  key?: string;
  icon: React.ReactNode;
  tooltipText?: string;
};

const IconButton: React.FC<Props> = ({ key, className, icon, tooltipText, ...props }) => {
  return tooltipText ? (
    <ButtonWithTooltip
      key={key}
      className={`transition-all text-zinc-400 hover:bg-zinc-700 hover:text-zinc-100 cursor-pointer ${className}`}
      variant="ghost"
      size="icon"
      tooltipText={tooltipText}
      {...props}>
      {icon}
    </ButtonWithTooltip>
  ) : (
    <Button
      key={key}
      className={`transition-all text-zinc-400 hover:bg-zinc-700 hover:text-zinc-100 cursor-pointer ${className}`}
      variant="ghost"
      size="icon"
      {...props}>
      {icon}
    </Button>
  );
};

export { IconButton };
