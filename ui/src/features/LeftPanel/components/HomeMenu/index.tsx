import { ChevronDown, Search } from "lucide-react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
  FlowLogo,
  IconButton,
} from "@flow/components";
import { config } from "@flow/config";
import { useOpenLink } from "@flow/hooks";
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";

import { AccountSetting, KeyboardSetting, WorkflowSetting, WorkspacesSetting } from "./components";

type Props = {};

const HomeMenu: React.FC<Props> = () => {
  const [, setDialogType] = useDialogType();
  const t = useT();
  const githubRepoUrl = config()?.githubRepoUrl;

  const handleGithubPageOpen = useOpenLink(githubRepoUrl ?? "");
  return (
    <DropdownMenu>
      <div className="flex justify-between items-center">
        <DropdownMenuTrigger className="flex justify-between items-center rounded py-1.5 px-2 bg-red-900/80 border border-transparent transition-colors hover:bg-transparent hover:border-red-900">
          <FlowLogo wrapperClassName="justify-start bg-opacity-75 rounded-md hover:bg-opacity-100 transition-colors" />
          <ChevronDown className="ml-2" size="12px" />
        </DropdownMenuTrigger>
        <div>
          <IconButton
            variant="ghost"
            size="icon"
            icon={<Search className="stroke-1" />}
            onClick={() => setDialogType("canvas-search")}
          />
        </div>
      </div>
      <DropdownMenuContent sideOffset={4} align="start" alignOffset={5} className="w-[275px]">
        <DropdownMenuLabel className="flex gap-2 text-zinc-400 justify-end items-center">
          <p>
            {t("Re:Earth Flow v")}
            {config()?.version ?? "X.X.X"}
          </p>
        </DropdownMenuLabel>
        <DropdownMenuSeparator className="bg-zinc-800" />
        <DropdownMenuGroup>
          <DropdownMenuItem onClick={() => setDialogType("welcome-init")}>
            {t("Home")}
            <DropdownMenuShortcut>⇧⌘H</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuSeparator className="bg-zinc-800" />
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
        {githubRepoUrl && (
          <DropdownMenuItem onClick={handleGithubPageOpen}>{t("GitHub")}</DropdownMenuItem>
        )}
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
