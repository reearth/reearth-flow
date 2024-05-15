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

const UserNavigation: React.FC = () => {
  const t = useT();
  const [, setDialogType] = useDialogType();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <div className="flex gap-2 mr-2">
          <Avatar className="h-9 w-9">
            <AvatarImage src="https://github.com/KaWaite.png" />
            <AvatarFallback>KW</AvatarFallback>
          </Avatar>
          <div className="self-center">
            <p className="text-zinc-400 font-extralight max-w-28 truncate transition-all delay-0 duration-500 hover:max-w-[30vw] hover:delay-500">
              KaWaite-007
            </p>
          </div>
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="text-zinc-300 w-[200px]"
        side="bottom"
        align="end"
        sideOffset={4}>
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
