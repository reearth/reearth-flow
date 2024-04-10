import { ChevronDown } from "lucide-react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
  FlowLogo,
} from "@flow/components";
import { useT } from "@flow/providers";

import { AccountSetting, KeyboardSetting, WorkflowSetting, WorkspacesSetting } from "./components";

type Props = {};

const HomeMenu: React.FC<Props> = () => {
  const t = useT();
  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="flex items-center [&>div]:data-[state=open]:bg-red-900">
        <FlowLogo wrapperClassName="justify-start bg-opacity-75 py-1.5 px-2 rounded-md hover:bg-opacity-100 transition-colors" />
        <ChevronDown className="ml-2" size="12px" />
      </DropdownMenuTrigger>
      <DropdownMenuContent sideOffset={4} align="start" alignOffset={5} className="w-[275px]">
        <DropdownMenuLabel className="flex gap-2 text-zinc-400 justify-end items-center">
          <p>{t("Re:Earth Flow v.1.14.2")}</p>
        </DropdownMenuLabel>
        <DropdownMenuSeparator className="bg-zinc-800" />
        <DropdownMenuGroup>
          <AccountSetting />
          <WorkspacesSetting />
          <WorkflowSetting />
        </DropdownMenuGroup>
        <DropdownMenuSeparator className="bg-zinc-800" />
        <DropdownMenuGroup>
          <KeyboardSetting />
          <DropdownMenuSub>
            <DropdownMenuSubTrigger>Invite users</DropdownMenuSubTrigger>
            <DropdownMenuPortal>
              <DropdownMenuSubContent sideOffset={2}>
                <DropdownMenuItem>Email</DropdownMenuItem>
                <DropdownMenuItem>Message</DropdownMenuItem>
                <DropdownMenuSeparator className="bg-zinc-800" />
                <DropdownMenuItem>More...</DropdownMenuItem>
              </DropdownMenuSubContent>
            </DropdownMenuPortal>
          </DropdownMenuSub>
        </DropdownMenuGroup>
        <DropdownMenuSeparator className="bg-zinc-800" />
        <DropdownMenuItem
          onClick={() =>
            window.open("https://github.com/reearth/reearth-flow", "_blank", "noopener")
          }>
          {t("GitHub")}
        </DropdownMenuItem>
        <DropdownMenuItem disabled>{t("Support (coming soon)")}</DropdownMenuItem>
        {/* <DropdownMenuItem disabled>API</DropdownMenuItem> */}
        <DropdownMenuSeparator className="bg-zinc-800" />
        <DropdownMenuItem>
          Log out
          {/* <DropdownMenuShortcut>⇧⌘Q</DropdownMenuShortcut> */}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export default HomeMenu;
