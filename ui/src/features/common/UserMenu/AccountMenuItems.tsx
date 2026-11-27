import {
  ArrowUpRightIcon,
  ChatCircleTextIcon,
  SignOutIcon,
  UserIcon,
} from "@phosphor-icons/react";

import { DropdownMenuItem, DropdownMenuSeparator } from "@flow/components";
import { config } from "@flow/config";
import { useAuth } from "@flow/lib/auth";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { dashboardAccountUrl, openLinkInNewTab } from "@flow/utils";

const rowClass = "gap-3 rounded-md px-3 py-[5px] font-light";

type Props = {
  onAccountSettingsOpen: () => void;
};

const AccountMenuItems: React.FC<Props> = ({ onAccountSettingsOpen }) => {
  const t = useT();
  const { logout: handleLogout } = useAuth();
  const { useGetMe } = useUser();
  const { me } = useGetMe();
  const { feedbackUrl } = config();
  // With a Dashboard, the account is managed there.
  const externalAccountUrl = dashboardAccountUrl();

  return (
    <>
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
        className={rowClass}
        onClick={
          externalAccountUrl
            ? openLinkInNewTab(externalAccountUrl)
            : onAccountSettingsOpen
        }>
        <UserIcon weight="light" />
        <p className="flex-1 truncate">{t("Account Settings")}</p>
        {externalAccountUrl && <ArrowUpRightIcon weight="light" />}
      </DropdownMenuItem>
      {feedbackUrl && (
        <DropdownMenuItem
          className={rowClass}
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
    </>
  );
};

export { AccountMenuItems };
