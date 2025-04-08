import {
  ArrowSquareOut,
  Broadcast,
  CaretDown,
  Keyboard,
  Rocket,
  SignOut,
  SneakerMove,
  SquaresFour,
  User,
} from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { useCallback, useState } from "react";

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
import { useShortcuts } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

type Props = {
  className?: string;
  dropdownPosition?: "left" | "right" | "bottom" | "top";
  dropdownAlign?: "start" | "center" | "end";
  dropdownPositionOffset?: number;
  dropdownAlignOffset?: number;
};

const HomeMenu: React.FC<Props> = ({
  // className,
  dropdownPosition = "right",
  dropdownAlign,
  dropdownAlignOffset,
  dropdownPositionOffset,
}) => {
  const t = useT();
  const navigate = useNavigate();
  const { workspaceId } = useParams({ strict: false });

  const { logout: handleLogout } = useAuth();

  const handleNavigationToDashboard = useCallback(
    (page: "projects" | "deployments" | "triggers" | "jobs") => () => {
      navigate({ to: `/workspaces/${workspaceId}/${page}` });
    },
    [workspaceId, navigate],
  );

  const [openAccountUpdateDialog, setOpenAccountUpdateDialog] = useState(false);
  const [openShortcutDialog, setOpenShortcutDialog] = useState(false);

  const { tosUrl, documentationUrl } = config();

  const handleTosPageOpen = openLinkInNewTab(tosUrl ?? "");
  const handleDocumentationPageOpen = openLinkInNewTab(documentationUrl ?? "");

  useShortcuts([
    {
      keyBinding: { key: "/", commandKey: true },
      callback: () => setOpenShortcutDialog((o) => !o),
    },
  ]);

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <div className="self-start h-full flex gap-2 items-center pl-4 pr-2 group cursor-pointer hover:bg-primary">
            <FlowLogo className="size-6 transition-all group-hover:text-[#46ce7c]" />
            <CaretDown weight="thin" />
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="min-w-[175px]"
          side={dropdownPosition}
          align={dropdownAlign}
          sideOffset={dropdownPositionOffset ?? 4}
          alignOffset={dropdownAlignOffset ?? 0}>
          <DropdownMenuLabel>{t("Dashboard")}</DropdownMenuLabel>
          {/* <DropdownMenuSeparator /> */}
          <DropdownMenuGroup>
            <DropdownMenuItem
              className="gap-3"
              onClick={handleNavigationToDashboard("projects")}>
              <SquaresFour weight="light" />
              <p>{t("Projects")}</p>
            </DropdownMenuItem>
            <DropdownMenuItem
              className="gap-3"
              onClick={handleNavigationToDashboard("deployments")}>
              <Rocket weight="light" />
              <p>{t("Deployments")}</p>
            </DropdownMenuItem>
            <DropdownMenuItem
              className="gap-3"
              onClick={handleNavigationToDashboard("triggers")}>
              <Broadcast weight="light" />
              <p>{t("Triggers")}</p>
            </DropdownMenuItem>
            <DropdownMenuItem
              className="gap-3"
              onClick={handleNavigationToDashboard("jobs")}>
              <SneakerMove weight="light" />
              <p>{t("Jobs")}</p>
            </DropdownMenuItem>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-3"
            onClick={() => setOpenAccountUpdateDialog(true)}>
            <User weight="light" />
            <p>{t("Account Settings")}</p>
          </DropdownMenuItem>
          <DropdownMenuItem
            className="gap-3"
            onClick={() => setOpenShortcutDialog(true)}>
            <Keyboard weight="light" />
            <p>{t("Keyboard Shortcuts")}</p>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          {tosUrl && (
            <DropdownMenuItem className="gap-3" onClick={handleTosPageOpen}>
              <ArrowSquareOut weight="light" />
              <p>{t("Terms of Service")}</p>
            </DropdownMenuItem>
          )}
          {documentationUrl && (
            <DropdownMenuItem
              className="gap-3"
              onClick={handleDocumentationPageOpen}>
              <ArrowSquareOut weight="light" />
              <p>{t("Documentation")}</p>
            </DropdownMenuItem>
          )}
          {/* <UserMenu className="w-full" /> */}
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-3 text-warning"
            onClick={handleLogout}>
            <SignOut weight="light" />
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
