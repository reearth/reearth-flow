import {
  ArrowSquareOutIcon,
  BookIcon,
  BroadcastIcon,
  CaretDownIcon,
  CopyrightIcon,
  FileIcon,
  GavelIcon,
  KeyboardIcon,
  RocketIcon,
  SignOutIcon,
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
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  FlowLogo,
} from "@flow/components";
import { config } from "@flow/config";
import { AccountUpdateDialog } from "@flow/features/common/UserMenu/AccountUpdateDialog";
import KeyboardShortcutDialog from "@flow/features/KeyboardShortcutDialog";
import { GENERAL_HOT_KEYS } from "@flow/global-constants";
import { useDoubleClick } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

import { AttributionsDialog } from "./AttributionsDialog";

type Props = {
  className?: string;
  dropdownPosition?: "left" | "right" | "bottom" | "top";
  dropdownAlign?: "start" | "center" | "end";
  dropdownPositionOffset?: number;
  dropdownAlignOffset?: number;
};

const HomeMenu: React.FC<Props> = ({
  // className,
  dropdownPosition = "bottom",
  dropdownAlign,
  dropdownAlignOffset,
  dropdownPositionOffset,
}) => {
  const t = useT();
  const navigate = useNavigate();
  const { workspaceId } = useParams({ strict: false });

  const { logout: handleLogout } = useAuth();

  const handleNavigationToDashboard = useCallback(
    (page: "projects" | "deployments" | "triggers" | "jobs" | "assets") => () =>
      navigate({ to: `/workspaces/${workspaceId}/${page}` }),
    [workspaceId, navigate],
  );

  const [openAccountUpdateDialog, setOpenAccountUpdateDialog] = useState(false);
  const [openShortcutDialog, setOpenShortcutDialog] = useState(false);
  const [openAttributionsDialog, setOpenAttributionsDialog] = useState(false);
  const [openDialog, setOpenDialog] = useState(false);

  const { tosUrl, documentationUrl } = config();

  const handleTosPageOpen = openLinkInNewTab(tosUrl ?? "");
  const handleDocumentationPageOpen = openLinkInNewTab(documentationUrl ?? "");

  useHotkeys(GENERAL_HOT_KEYS, () => setOpenShortcutDialog(true));

  const [, handleDoubleClick] = useDoubleClick(() => {
    setOpenDialog((o) => !o);
  }, handleNavigationToDashboard("projects"));

  return (
    <>
      <DropdownMenu open={openDialog} onOpenChange={setOpenDialog}>
        <DropdownMenuTrigger asChild onDoubleClick={handleDoubleClick}>
          <div className="group flex cursor-pointer items-center gap-1 self-center rounded-md p-1 hover:bg-primary">
            <FlowLogo className="size-7 transition-all group-hover:text-[#46ce7c]" />
            <CaretDownIcon weight="thin" size={12} />
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="min-w-[175px] bg-primary/50 backdrop-blur"
          side={dropdownPosition}
          align={dropdownAlign}
          sideOffset={dropdownPositionOffset ?? 8}
          alignOffset={dropdownAlignOffset ?? 0}>
          <DropdownMenuLabel>{t("Dashboard")}</DropdownMenuLabel>
          {/* <DropdownMenuSeparator /> */}
          <DropdownMenuGroup>
            <DropdownMenuItem
              className="gap-3"
              onClick={handleNavigationToDashboard("projects")}>
              <SquaresFourIcon weight="light" />
              <p>{t("Projects")}</p>
            </DropdownMenuItem>
            <DropdownMenuItem
              className="gap-3"
              onClick={handleNavigationToDashboard("deployments")}>
              <RocketIcon weight="light" />
              <p>{t("Deployments")}</p>
            </DropdownMenuItem>
            <DropdownMenuItem
              className="gap-3"
              onClick={handleNavigationToDashboard("triggers")}>
              <BroadcastIcon weight="light" />
              <p>{t("Triggers")}</p>
            </DropdownMenuItem>
            <DropdownMenuItem
              className="gap-3"
              onClick={handleNavigationToDashboard("jobs")}>
              <SneakerMoveIcon weight="light" />
              <p>{t("Jobs")}</p>
            </DropdownMenuItem>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-3"
            onClick={handleNavigationToDashboard("assets")}>
            <FileIcon weight="light" />
            <p>{t("Workspace Assets")}</p>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-3"
            onClick={() => setOpenAccountUpdateDialog(true)}>
            <UserIcon weight="light" />
            <p>{t("Account Settings")}</p>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-3"
            onClick={() => setOpenShortcutDialog(true)}>
            <KeyboardIcon weight="light" />
            <p>{t("Keyboard Shortcuts")}</p>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          {tosUrl && (
            <DropdownMenuItem className="gap-3" onClick={handleTosPageOpen}>
              <GavelIcon weight="light" />
              <div className="flex items-center gap-1">
                <p>{t("Terms of Service")}</p>
                <ArrowSquareOutIcon className="size-4" />
              </div>
            </DropdownMenuItem>
          )}
          {documentationUrl && (
            <DropdownMenuItem
              className="gap-3"
              onClick={handleDocumentationPageOpen}>
              <BookIcon weight="light" />
              <div className="flex items-center gap-1">
                <p>{t("Documentation")}</p>
                <ArrowSquareOutIcon weight="light" />
              </div>
            </DropdownMenuItem>
          )}
          <DropdownMenuItem
            className="gap-3"
            onClick={() => setOpenAttributionsDialog(true)}>
            <CopyrightIcon weight="light" />
            <p>{t("Attributions")}</p>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-3 text-warning"
            onClick={handleLogout}>
            <SignOutIcon weight="light" />
            <p>{t("Log Out")}</p>
          </DropdownMenuItem>
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
