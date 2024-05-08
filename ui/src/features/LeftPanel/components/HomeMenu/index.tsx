import { DashboardIcon } from "@radix-ui/react-icons";
import { useNavigate } from "@tanstack/react-router";
import { ChevronDown, Search } from "lucide-react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuSeparator,
  // DropdownMenuShortcut,
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
import { useCurrentWorkspace, useDialogType } from "@flow/stores";

import { AccountSetting, KeyboardSetting, WorkflowSetting, WorkspacesSetting } from "./components";

type Props = {};

const HomeMenu: React.FC<Props> = () => {
  const [, setDialogType] = useDialogType();
  const t = useT();
  const { githubRepoUrl, brandName, version } = config();
  const [currentWorkspace] = useCurrentWorkspace();
  const navigate = useNavigate({ from: "/project/$projectId" });

  const handleGithubPageOpen = useOpenLink(githubRepoUrl ?? "");
  return (
    <div className="flex justify-between items-center">
      <DropdownMenu>
        <DropdownMenuTrigger className="flex justify-between items-center rounded px-2 transition-colors group">
          <FlowLogo wrapperClassName="justify-start bg-opacity-75 bg-red-800/50 p-2 rounded transition-colors border border-transparent group-hover:bg-transparent group-hover:border-red-900" />
          <ChevronDown className="ml-2 group-hover:text-zinc-200" size="12px" />
        </DropdownMenuTrigger>

        <DropdownMenuContent sideOffset={4} align="start" alignOffset={5} className="w-[275px]">
          <DropdownMenuLabel className="flex gap-2 text-zinc-400 justify-end items-center">
            <p>
              {brandName ?? t("Re:Earth Flow v")}
              {version}
            </p>
          </DropdownMenuLabel>
          <DropdownMenuSeparator className="bg-zinc-800" />
          <DropdownMenuGroup>
            <DropdownMenuItem
              className="gap-2"
              onClick={() => navigate({ to: `/workspace/${currentWorkspace?.id}` })}>
              <DashboardIcon />
              {t("Dashboard")}
              {/* <DropdownMenuShortcut>⇧⌘H</DropdownMenuShortcut> */}
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
      <div>
        <IconButton
          variant="ghost"
          size="icon"
          icon={<Search className="stroke-1" />}
          onClick={() => setDialogType("canvas-search")}
        />
      </div>
    </div>
  );
};

export default HomeMenu;
