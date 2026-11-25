import {
  ArrowUpRightIcon,
  ChatCircleTextIcon,
  SignOutIcon,
  UserIcon,
} from "@phosphor-icons/react";
import { useState } from "react";

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

  // With a Dashboard, the account is managed there.
  const externalAccountUrl = dashboardAccountUrl();
  const { feedbackUrl } = config();
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
          className="flex w-[220px] flex-col gap-0.5"
          side={dropdownPosition}
          align={dropdownAlign}
          sideOffset={dropdownOffset ?? 4}>
          <div className="flex flex-col gap-0.5 px-3 py-1.5">
            <p className="truncate text-sm">{me?.name ?? me?.email}</p>
            {me?.name && me.email && (
              <p className="truncate text-xs font-light text-muted-foreground">
                {me.email}
              </p>
            )}
          </div>
          <DropdownMenuSeparator className="mx-0 my-0" />
          <DropdownMenuItem
            className="gap-3 rounded-md px-3 py-[5px] font-light"
            onClick={
              externalAccountUrl
                ? openLinkInNewTab(externalAccountUrl)
                : () => setOpenAccountUpdateDialog(true)
            }>
            <UserIcon weight="light" />
            <p className="flex-1 truncate">{t("Account Settings")}</p>
            {externalAccountUrl && <ArrowUpRightIcon weight="light" />}
          </DropdownMenuItem>
          {feedbackUrl && (
            <DropdownMenuItem
              className="gap-3 rounded-md px-3 py-[5px] font-light"
              onClick={openLinkInNewTab(feedbackUrl)}>
              <ChatCircleTextIcon weight="light" />
              <p className="flex-1 truncate">{t("Share Feedback")}</p>
              <ArrowUpRightIcon weight="light" />
            </DropdownMenuItem>
          )}
          <DropdownMenuSeparator className="mx-0 my-0" />
          <DropdownMenuItem
            className="gap-3 rounded-md px-3 py-[5px] text-warning"
            onClick={handleLogout}>
            <SignOutIcon weight="light" className="text-warning" />
            <p className="flex-1 truncate">{t("Log Out")}</p>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      {openAccountUpdateDialog && (
        <AccountUpdateDialog
          isOpen={openAccountUpdateDialog}
          onOpenChange={setOpenAccountUpdateDialog}
        />
      )}
    </>
  );
};

export { UserMenu };
