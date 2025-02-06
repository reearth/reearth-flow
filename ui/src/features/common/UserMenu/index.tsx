import { Keyboard, SignOut, User } from "@phosphor-icons/react";
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
          className="w-[200px]"
          side={dropdownPosition ?? "bottom"}
          align="end"
          sideOffset={dropdownOffset ?? 4}>
          <div className="mb-2 rounded px-2 py-1">
            <p className="text-xs font-thin">{t("Username: ")}</p>
            <p className="truncate text-sm font-light">
              {me?.name ?? me?.email}
            </p>
          </div>
          <DropdownMenuItem
            className="gap-2"
            onClick={() => setOpenAccountUpdateDialog(true)}>
            <User weight="thin" />
            <p>{t("Account settings")}</p>
          </DropdownMenuItem>
          <DropdownMenuItem
            className="gap-2"
            onClick={() => setOpenShortcutDialog(true)}>
            <Keyboard weight="thin" />
            <p>{t("Keyboard shortcuts")}</p>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          {tosUrl && (
            <DropdownMenuItem onClick={handleTosPageOpen}>
              <p>{t("Terms of Service")}</p>
            </DropdownMenuItem>
          )}
          {documentationUrl && (
            <DropdownMenuItem onClick={handleDocumentationPageOpen}>
              <p>{t("Documentation")}</p>
            </DropdownMenuItem>
          )}
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={handleLogout} className="gap-2">
            <SignOut className="size-[15px] stroke-1" />
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
