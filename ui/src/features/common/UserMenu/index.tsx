import { ArrowSquareOut, Keyboard, SignOut, User } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Avatar,
  AvatarFallback,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@flow/components";
import { config } from "@flow/config";
import KeyboardShortcutDialog from "@flow/features/KeyboardShortcutDialog";
import { useShortcuts } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { openLinkInNewTab } from "@flow/utils";

import { AccountUpdateDialog } from "./AccountUpdateDialog";

type Props = {
  className?: string;
  dropdownPosition?: "left" | "right" | "bottom" | "top";
  dropdownOffset?: number;
};

const UserMenu: React.FC<Props> = ({
  className,
  dropdownPosition,
  dropdownOffset,
}) => {
  const t = useT();
  const { logout: handleLogout } = useAuth();
  const { useGetMe } = useUser();
  const { me } = useGetMe();

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
        <DropdownMenuTrigger>
          <div className={`flex items-center gap-1 ${className}`}>
            <Avatar className="size-7">
              <AvatarFallback>
                {me?.name ? me.name.charAt(0).toUpperCase() : "?"}
              </AvatarFallback>
            </Avatar>
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          side={dropdownPosition ?? "right"}
          align="start"
          sideOffset={dropdownOffset ?? 4}>
          <div className="mb-2 rounded p-2">
            <p className="text-xs font-thin">{t("Username: ")}</p>
            <p className="truncate px-2 text-sm">{me?.name ?? me?.email}</p>
          </div>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-2"
            onClick={() => setOpenAccountUpdateDialog(true)}>
            <User weight="light" />
            <p>{t("Account settings")}</p>
          </DropdownMenuItem>
          <DropdownMenuItem
            className="gap-2"
            onClick={() => setOpenShortcutDialog(true)}>
            <Keyboard weight="light" />
            <p>{t("Keyboard shortcuts")}</p>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          {tosUrl && (
            <DropdownMenuItem className="gap-2" onClick={handleTosPageOpen}>
              <ArrowSquareOut weight="light" />
              <p>{t("Terms of Service")}</p>
            </DropdownMenuItem>
          )}
          {documentationUrl && (
            <DropdownMenuItem
              className="gap-2"
              onClick={handleDocumentationPageOpen}>
              <ArrowSquareOut weight="light" />
              <p>{t("Documentation")}</p>
            </DropdownMenuItem>
          )}
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-2 text-warning"
            onClick={handleLogout}>
            <SignOut weight="light" />
            <p>{t("Log out")}</p>
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
