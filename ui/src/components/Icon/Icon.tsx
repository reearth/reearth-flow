import { FC, useMemo } from "react";

import Icons from "./icons";

export type IconName = keyof typeof Icons;

export type IconProps = {
  icon: IconName;
  size?: "large" | "normal" | "small" | number;
  color?: string;
  className?: string;
  ariaLabel?: string;
  dataTestId?: string;
};

const iconSizes = { small: 16, normal: 24, large: 32 };

const Icon: FC<IconProps> = ({
  icon,
  size = "normal",
  color,
  className,
  ariaLabel,
  dataTestId = `icon-${icon}`,
}) => {
  const SvgIcon = useMemo(() => Icons[icon as IconName], [icon]);

  if (!SvgIcon) return null;

  const px = typeof size === "number" ? size : iconSizes[size];

  return (
    <SvgIcon
      style={{ width: px, height: px, color }}
      className={className}
      aria-label={ariaLabel}
      aria-hidden={!ariaLabel ? "true" : undefined}
      data-testid={dataTestId}
    />
  );
};

export { Icon };
