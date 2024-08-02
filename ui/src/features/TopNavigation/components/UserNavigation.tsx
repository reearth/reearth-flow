import { CaretDown, Keyboard, SignOut, User } from "@phosphor-icons/react";

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
import { useOpenLink } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useDialogType } from "@flow/stores";

type Props = {
  className?: string;
  iconOnly?: boolean;
  dropdownPosition?: "left" | "right" | "bottom" | "top";
  dropdownOffset?: number;
};

const UserNavigation: React.FC<Props> = ({
  className,
  iconOnly,
  dropdownPosition,
  dropdownOffset,
}) => {
  const t = useT();
  const [, setDialogType] = useDialogType();
  const { logout: handleLogout, user } = useAuth();
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  const { tosUrl, documentationUrl } = config();

  const handleTosPageOpen = useOpenLink(tosUrl ?? "");
  const handleDocumentationPageOpen = useOpenLink(documentationUrl ?? "");

  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <div className={`mr-2 flex gap-2 ${className}`}>
          <Avatar className="size-8">
            <AvatarImage src={user?.picture} />
            <AvatarFallback>
              {me?.name ? me.name.charAt(0).toUpperCase() : "F"}
            </AvatarFallback>
          </Avatar>
          {!iconOnly ? (
            <div className="flex items-center gap-2 self-center">
              <p className="max-w-28 truncate text-sm font-extralight transition-all delay-0 duration-500 hover:max-w-[30vw] hover:delay-500">
                {me?.name ? me.name : "User"}
              </p>
              <CaretDown className="w-[12px]" weight="thin" />
            </div>
          ) : null}
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="w-[200px]"
        side={dropdownPosition ?? "bottom"}
        align="end"
        sideOffset={dropdownOffset ?? 4}
      >
        {/* <DropdownMenuLabel>My Account</DropdownMenuLabel> */}
        <DropdownMenuItem
          className="gap-2"
          onClick={() => setDialogType("account-settings")}
        >
          <User weight="thin" />
          <p>{t("Account settings")}</p>
        </DropdownMenuItem>
        <DropdownMenuItem
          className="gap-2"
          onClick={() => setDialogType("keyboard-instructions")}
        >
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
  );
};

export { UserNavigation };
