import {
  ArrowUpRightIcon,
  BookIcon,
  BroadcastIcon,
  CaretDownIcon,
  FileIcon,
  FileTextIcon,
  KeyboardIcon,
  PaletteIcon,
  RocketIcon,
  SneakerMoveIcon,
  SquaresFourIcon,
  UserIcon,
} from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { useCallback, useState } from "react";
import { useHotkeys } from "react-hotkeys-hook";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
  FlowLogo,
} from "@flow/components";
import { config } from "@flow/config";
import { AttributionsDialog } from "@flow/features/common/AttributionsDialog";
import { LegalMenuItems } from "@flow/features/common/LegalMenuItems";
import { ThemeLanguageMenu } from "@flow/features/common/ThemeLanguageMenu";
import { AccountMenuItems } from "@flow/features/common/UserMenu/AccountMenuItems";
import { AccountUpdateDialog } from "@flow/features/common/UserMenu/AccountUpdateDialog";
import KeyboardShortcutDialog from "@flow/features/KeyboardShortcutDialog";
import { GENERAL_HOT_KEYS } from "@flow/global-constants";
import { useDoubleClick } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

const rowClass = "gap-3 rounded-md px-3 py-[5px] font-light";
const subTriggerClass = `${rowClass} [&>svg:last-child]:text-muted-foreground`;
const subContentClass = "flex w-[220px] flex-col gap-0.5 rounded-lg";

type Props = {
  dropdownPosition?: "left" | "right" | "bottom" | "top";
  dropdownAlign?: "start" | "center" | "end";
  dropdownPositionOffset?: number;
  dropdownAlignOffset?: number;
};

const HomeMenu: React.FC<Props> = ({
  dropdownPosition = "bottom",
  dropdownAlign = "start",
  dropdownAlignOffset,
  dropdownPositionOffset,
}) => {
  const t = useT();
  const navigate = useNavigate();
  const { workspaceId } = useParams({ strict: false });
  const { documentationUrl } = config();

  const handleNavigationToDashboard = useCallback(
    (page: "projects" | "deployments" | "triggers" | "jobs" | "assets") => () =>
      navigate({ to: `/workspaces/${workspaceId}/${page}` }),
    [workspaceId, navigate],
  );

  const [openAccountUpdateDialog, setOpenAccountUpdateDialog] = useState(false);
  const [openShortcutDialog, setOpenShortcutDialog] = useState(false);
  const [openAttributionsDialog, setOpenAttributionsDialog] = useState(false);
  const [openDialog, setOpenDialog] = useState(false);

  useHotkeys(GENERAL_HOT_KEYS, () => setOpenShortcutDialog(true));

  const [, handleDoubleClick] = useDoubleClick(() => {
    setOpenDialog((o) => !o);
  }, handleNavigationToDashboard("projects"));

  const navItems = [
    {
      page: "projects",
      label: t("Projects"),
      icon: <SquaresFourIcon weight="light" />,
    },
    {
      page: "deployments",
      label: t("Deployments"),
      icon: <RocketIcon weight="light" />,
    },
    {
      page: "triggers",
      label: t("Triggers"),
      icon: <BroadcastIcon weight="light" />,
    },
    {
      page: "jobs",
      label: t("Jobs"),
      icon: <SneakerMoveIcon weight="light" />,
    },
    {
      page: "assets",
      label: t("Workspace Assets"),
      icon: <FileIcon weight="light" />,
    },
  ] as const;

  return (
    <>
      <DropdownMenu open={openDialog} onOpenChange={setOpenDialog}>
        <DropdownMenuTrigger
          nativeButton={false}
          onDoubleClick={handleDoubleClick}
          render={
            <div
              className={`group flex cursor-pointer items-center gap-1 rounded-md p-1 hover:bg-accent ${openDialog ? "bg-accent" : ""}`}>
              <FlowLogo className="size-7 transition-all group-hover:text-[#46ce7c]" />
              <CaretDownIcon
                weight="light"
                size={12}
                className="text-muted-foreground"
              />
            </div>
          }
        />
        <DropdownMenuContent
          className="flex w-56 flex-col gap-0.5"
          side={dropdownPosition}
          align={dropdownAlign}
          sideOffset={dropdownPositionOffset ?? 8}
          alignOffset={dropdownAlignOffset ?? 0}>
          {navItems.map(({ page, label, icon }) => (
            <DropdownMenuItem
              key={page}
              className={rowClass}
              onClick={handleNavigationToDashboard(page)}>
              {icon}
              <p className="flex-1 truncate">{label}</p>
            </DropdownMenuItem>
          ))}
          <DropdownMenuSeparator className="mx-0 my-0" />
          <DropdownMenuSub>
            <DropdownMenuSubTrigger className={subTriggerClass}>
              <UserIcon weight="light" />
              <p className="flex-1 truncate">{t("Account")}</p>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className={subContentClass}>
              <AccountMenuItems
                onAccountSettingsOpen={() => setOpenAccountUpdateDialog(true)}
              />
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSeparator className="mx-0 my-0" />
          {documentationUrl && (
            <DropdownMenuItem
              className={rowClass}
              onClick={openLinkInNewTab(documentationUrl)}>
              <BookIcon weight="light" />
              <p className="flex-1 truncate">{t("Documentation")}</p>
              <ArrowUpRightIcon weight="light" />
            </DropdownMenuItem>
          )}
          <DropdownMenuItem
            className={rowClass}
            onClick={() => setOpenShortcutDialog(true)}>
            <KeyboardIcon weight="light" />
            <p className="flex-1 truncate">{t("Keyboard Shortcuts")}</p>
          </DropdownMenuItem>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger className={subTriggerClass}>
              <PaletteIcon weight="light" />
              <p className="flex-1 truncate">{t("Theme & Language")}</p>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className={subContentClass}>
              <ThemeLanguageMenu />
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger className={subTriggerClass}>
              <FileTextIcon weight="light" />
              <p className="flex-1 truncate">{t("Legal & about")}</p>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className={subContentClass}>
              <LegalMenuItems
                onAttributionsOpen={() => setOpenAttributionsDialog(true)}
              />
            </DropdownMenuSubContent>
          </DropdownMenuSub>
        </DropdownMenuContent>
      </DropdownMenu>
      {openAccountUpdateDialog && (
        <AccountUpdateDialog
          isOpen={openAccountUpdateDialog}
          onOpenChange={setOpenAccountUpdateDialog}
        />
      )}
      {openAttributionsDialog && (
        <AttributionsDialog
          isOpen={openAttributionsDialog}
          onOpenChange={setOpenAttributionsDialog}
        />
      )}
      {openShortcutDialog && (
        <KeyboardShortcutDialog
          isOpen={openShortcutDialog}
          onOpenChange={setOpenShortcutDialog}
        />
      )}
    </>
  );
};

export default HomeMenu;
