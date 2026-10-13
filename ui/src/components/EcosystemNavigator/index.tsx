import { useState } from "react";

import { config } from "@flow/config";
import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";

import { CmsLogo, Icon } from "../Icon";
import type { IconName } from "../Icon";
import { Popover, PopoverContent, PopoverTrigger } from "../Popover";

type Props = {
  className?: string;
  iconSize?: number;
  side?: "top" | "bottom" | "left" | "right";
  align?: "start" | "center" | "end";
  sideOffset?: number;
  alignOffset?: number;
};

const EcosystemNavigator: React.FC<Props> = ({
  className,
  iconSize = 18,
  side = "bottom",
  align = "start",
  sideOffset,
  alignOffset,
}) => {
  const t = useT();
  const [open, setOpen] = useState(false);

  const {
    reearthHomeUrl,
    reearthDashboardUrl,
    reearthVisualizerUrl,
    reearthCmsUrl,
    navaraUrl,
    reearthTerrainUrl,
    reearthBuildingsUrl,
    reearthPapersUrl,
    reearthCommunityUrl,
  } = config();

  const handleLinkClick = () => setOpen(false);

  const dataServices = [
    {
      name: "Terrain",
      description: t("Open terrain tiles for 3D globes"),
      href: reearthTerrainUrl,
    },
    {
      name: "Buildings",
      description: t("Open 3D buildings tiles for the world"),
      href: reearthBuildingsUrl,
    },
    {
      name: "Papers",
      description: t("Open Map Tile Service"),
      href: reearthPapersUrl,
    },
  ];

  const others: { name: string; icon: IconName; href?: string }[] = [
    { name: t("Re:Earth Home"), icon: "house", href: reearthHomeUrl },
    {
      name: t("Community"),
      icon: "community",
      href: reearthCommunityUrl,
    },
  ];

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        className={cn(
          "flex cursor-pointer items-center rounded p-1 transition-colors hover:bg-primary focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden data-popup-open:bg-primary",
          className,
        )}
        aria-label={t("Re:Earth products")}>
        <Icon icon="dotsNine" size={iconSize} />
      </PopoverTrigger>
      <PopoverContent
        className="flex w-auto max-w-[312px] flex-col gap-3 rounded-lg p-4 shadow-[0_6px_16px_0_rgba(0,0,0,0.35)] backdrop-blur-md"
        side={side}
        align={align}
        sideOffset={sideOffset}
        alignOffset={alignOffset}>
        <SectionLabel>{t("Re:Earth products")}</SectionLabel>
        <div className="flex gap-2">
          <ProductLink
            name="Dashboard"
            href={reearthDashboardUrl}
            tintClassName="bg-[rgba(255,255,155,0.14)]"
            onClick={handleLinkClick}>
            <Icon icon="reearthDashboard" size={48} />
          </ProductLink>
          <ProductLink
            name="Visualizer"
            href={reearthVisualizerUrl}
            tintClassName="bg-[rgba(255,107,107,0.14)]"
            onClick={handleLinkClick}>
            <Icon icon="reearthVisualizer" size={48} />
          </ProductLink>
          <ProductLink
            name="CMS"
            href={reearthCmsUrl}
            tintClassName="bg-[rgba(255,197,61,0.14)]"
            onClick={handleLinkClick}>
            <CmsLogo className="size-12 text-[#F7B502]" />
          </ProductLink>
        </div>
        <Divider />
        <SectionLabel>{t("Map engine")}</SectionLabel>
        <ExternalLink
          className="flex w-full items-center gap-3 rounded-lg p-2 hover:bg-primary"
          href={navaraUrl}
          onClick={handleLinkClick}>
          <Icon icon="navara" size={64} />
          <div className="flex min-w-0 flex-1 flex-col">
            <LinkName name="Navara" />
            <LinkDescription className="wrap-break-word whitespace-normal">
              {t("3D map engine, MapLibre family")}
            </LinkDescription>
          </div>
        </ExternalLink>
        <Divider />
        <SectionLabel>{t("Re:Earth data services")}</SectionLabel>
        <div className="flex flex-col gap-2">
          {dataServices.map(({ name, description, href }) => (
            <ExternalLink
              key={name}
              className="group flex w-fit flex-col"
              href={href}
              onClick={handleLinkClick}>
              <LinkName
                className="-mx-1 rounded px-1 py-0.5 group-hover:bg-primary"
                name={name}
              />
              <LinkDescription>{description}</LinkDescription>
            </ExternalLink>
          ))}
        </div>
        <Divider />
        <SectionLabel>{t("Other")}</SectionLabel>
        <div className="flex gap-2">
          {others.map(({ name, icon, href }) => (
            <ExternalLink
              key={name}
              className="flex items-center gap-2 rounded-full bg-primary py-1.5 pr-3 pl-2.5 transition-colors hover:bg-accent hover:drop-shadow-[1px_2px_0.5px_rgba(36,39,43,0.31)]"
              href={href}
              onClick={handleLinkClick}>
              <Icon icon={icon} size={16} />
              <span className="text-xs font-light">{name}</span>
            </ExternalLink>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
};

const SectionLabel: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => <p className="text-xs text-muted-foreground">{children}</p>;

const Divider: React.FC = () => <div className="h-px w-full bg-border" />;

// A link without a URL renders as disabled. The Docker config template fills
// unset variables with "", so an empty string counts as no URL.
const ExternalLink: React.FC<{
  className?: string;
  href?: string;
  children: React.ReactNode;
  onClick: () => void;
}> = ({ className, href, children, onClick }) => (
  <a
    className={cn(
      "transition-colors focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden",
      !href && "pointer-events-none opacity-50",
      className,
    )}
    href={href || undefined}
    target="_blank"
    rel="noopener noreferrer"
    aria-disabled={!href || undefined}
    onClick={onClick}>
    {children}
  </a>
);

const ProductLink: React.FC<{
  name: string;
  href?: string;
  tintClassName: string;
  children: React.ReactNode;
  onClick: () => void;
}> = ({ name, href, tintClassName, children, onClick }) => (
  <ExternalLink
    className="flex w-[88px] flex-col items-center justify-center gap-1 rounded-lg p-1 hover:bg-primary"
    href={href}
    onClick={onClick}>
    <div className={cn("rounded-xl p-2", tintClassName)}>{children}</div>
    <span className="max-w-full truncate text-sm leading-[18px]">{name}</span>
  </ExternalLink>
);

const LinkName: React.FC<{ className?: string; name: string }> = ({
  className,
  name,
}) => (
  <div className={cn("flex items-center gap-1", className)}>
    <span className="text-sm leading-[18px] font-medium">{name}</span>
    <Icon className="text-muted-foreground" icon="arrowUpRight" size={12} />
  </div>
);

const LinkDescription: React.FC<{
  className?: string;
  children: React.ReactNode;
}> = ({ className, children }) => (
  <p
    className={cn(
      "text-xs font-light whitespace-nowrap text-muted-foreground",
      className,
    )}>
    {children}
  </p>
);

export { EcosystemNavigator };
