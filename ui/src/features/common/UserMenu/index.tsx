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
          <div className={`flex size-8 items-center gap-1 ${className}`}>
            <Avatar className="size-full">
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
            onClick={() => setOpenAccountUpdateDialog(true)}>
            <p>{t("Account Settings")}</p>
            <User weight="light" />
          </DropdownMenuItem>
          <DropdownMenuItem
            className="justify-between gap-4"
            onClick={() => setOpenShortcutDialog(true)}>
            <p>{t("Keyboard Shortcuts")}</p>
            <Keyboard weight="light" />
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          {tosUrl && (
            <DropdownMenuItem
              className="justify-between gap-4"
              onClick={handleTosPageOpen}>
              <p>{t("Terms of Service")}</p>
              <ArrowSquareOut weight="light" />
            </DropdownMenuItem>
          )}
          {documentationUrl && (
            <DropdownMenuItem
              className="justify-between gap-4"
              onClick={handleDocumentationPageOpen}>
              <p>{t("Documentation")}</p>
              <ArrowSquareOut weight="light" />
            </DropdownMenuItem>
          )}
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="justify-between gap-4 text-warning"
            onClick={handleLogout}>
            <p>{t("Log Out")}</p>
            <SignOut weight="light" />
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
