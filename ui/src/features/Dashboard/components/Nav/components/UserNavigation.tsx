import { KeyboardIcon, PersonIcon } from "@radix-ui/react-icons";
import { LogOut } from "lucide-react";

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
import { useT } from "@flow/providers";
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
  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <div className={`flex gap-2 mr-2 ${className}`}>
          <Avatar className="h-8 w-8">
            <AvatarImage src="https://github.com/KaWaite.png" />
            <AvatarFallback>KW</AvatarFallback>
          </Avatar>
          {!iconOnly && (
            <div className="self-center">
              <p className="text-zinc-400 text-sm font-extralight max-w-28 truncate transition-all delay-0 duration-500 hover:max-w-[30vw] hover:delay-500">
                KaWaite-007
              </p>
            </div>
          )}
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="text-zinc-300 w-[200px]"
        side={dropdownPosition ?? "bottom"}
        align="end"
        sideOffset={dropdownOffset ?? 4}>
        {/* <DropdownMenuLabel>My Account</DropdownMenuLabel> */}
        <DropdownMenuItem className="gap-2" onClick={() => setDialogType("account-settings")}>
          <PersonIcon />
          <p>{t("Account settings")}</p>
        </DropdownMenuItem>
        <DropdownMenuItem className="gap-2" onClick={() => setDialogType("keyboard-instructions")}>
          <KeyboardIcon />
          <p>{t("Keyboard shortcuts")}</p>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem className="gap-2">
          <LogOut className="w-[15px] h-[15px] stroke-1" />
          <p>{t("Log out")}</p>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { UserNavigation };
