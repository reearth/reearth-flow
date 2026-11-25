import {
  ArrowUpRightIcon,
  BookIcon,
  ChatCircleTextIcon,
  FileTextIcon,
  KeyboardIcon,
  SignOutIcon,
  UserIcon,
} from "@phosphor-icons/react";
import { useState } from "react";
import { useHotkeys } from "react-hotkeys-hook";

import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@flow/components";
import { config } from "@flow/config";
import KeyboardShortcutDialog from "@flow/features/KeyboardShortcutDialog";
import { GENERAL_HOT_KEYS } from "@flow/global-constants";
import { useAuth } from "@flow/lib/auth";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { dashboardAccountUrl, openLinkInNewTab } from "@flow/utils";

import { AccountUpdateDialog } from "./AccountUpdateDialog";

type Props = {
  className?: string;
  dropdownPosition?: "left" | "right" | "bottom" | "top";
  dropdownAlign?: "start" | "center" | "end";
  dropdownOffset?: number;
};

const UserMenu: React.FC<Props> = ({
  className,
  dropdownPosition = "right",
  dropdownAlign,
  dropdownOffset = 10,
}) => {
  const t = useT();
  const { logout: handleLogout } = useAuth();
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  const [openAccountUpdateDialog, setOpenAccountUpdateDialog] = useState(false);
  const [openShortcutDialog, setOpenShortcutDialog] = useState(false);

  const { tosUrl, documentationUrl, feedbackUrl } = config();

  const handleTosPageOpen = openLinkInNewTab(tosUrl ?? "");
  const handleDocumentationPageOpen = openLinkInNewTab(documentationUrl ?? "");
  // With a Dashboard, the account is managed there.
  const externalAccountUrl = dashboardAccountUrl();
  useHotkeys(GENERAL_HOT_KEYS, () => setOpenShortcutDialog(true));
  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger>
          <div className={`flex size-8 items-center gap-1 ${className}`}>
            <Avatar className="size-full">
              {me?.photoURL && <AvatarImage src={me.photoURL} alt={me.name} />}
              <AvatarFallback>
                {me?.name ? me.name.charAt(0).toUpperCase() : "?"}
              </AvatarFallback>
            </Avatar>
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="min-w-[175px]"
          side={dropdownPosition}
          align={dropdownAlign}
          sideOffset={dropdownOffset ?? 4}>
          <div className="mb-2 rounded p-2">
            <p className="text-xs font-thin">{t("Username: ")}</p>
            <p className="truncate px-2 text-sm">{me?.name ?? me?.email}</p>
          </div>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="justify-between gap-4"
            onClick={
              externalAccountUrl
                ? openLinkInNewTab(externalAccountUrl)
                : () => setOpenAccountUpdateDialog(true)
            }>
            <div className="flex items-center gap-3 whitespace-nowrap">
              <UserIcon weight="light" />
              <p>{t("Account Settings")}</p>
            </div>
            {externalAccountUrl && <ArrowUpRightIcon weight="light" />}
          </DropdownMenuItem>
          <DropdownMenuItem
            className="justify-between gap-4"
            disabled={!feedbackUrl}
            onClick={feedbackUrl ? openLinkInNewTab(feedbackUrl) : () => {}}>
            <div className="flex items-center gap-3 whitespace-nowrap">
              <ChatCircleTextIcon weight="light" />
              <p>{t("Share Feedback")}</p>
            </div>
            <ArrowUpRightIcon weight="light" />
          </DropdownMenuItem>
          <DropdownMenuItem
            className="justify-between gap-4"
            onClick={() => setOpenShortcutDialog(true)}>
            <div className="flex items-center gap-3 whitespace-nowrap">
              <KeyboardIcon weight="light" />
              <p>{t("Keyboard Shortcuts")}</p>
            </div>
            <span className="size-4 shrink-0" />
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          {tosUrl && (
            <DropdownMenuItem
              className="justify-between gap-4"
              onClick={handleTosPageOpen}>
              <div className="flex items-center gap-3 whitespace-nowrap">
                <FileTextIcon weight="light" />
                <p>{t("Terms of Service")}</p>
              </div>
              <ArrowUpRightIcon weight="light" />
            </DropdownMenuItem>
          )}
          {documentationUrl && (
            <DropdownMenuItem
              className="justify-between gap-4"
              onClick={handleDocumentationPageOpen}>
              <div className="flex items-center gap-3 whitespace-nowrap">
                <BookIcon weight="light" />
                <p>{t("Documentation")}</p>
              </div>
              <ArrowUpRightIcon weight="light" />
            </DropdownMenuItem>
          )}
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="justify-between gap-4 text-warning"
            onClick={handleLogout}>
            <div className="flex items-center gap-3 whitespace-nowrap">
              <SignOutIcon weight="light" className="text-warning" />
              <p>{t("Log Out")}</p>
            </div>
            <span className="size-4 shrink-0" />
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

export { UserMenu };
