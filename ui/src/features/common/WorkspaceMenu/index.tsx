import { ArrowUpRightIcon, PlusIcon } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
  ScrollArea,
} from "@flow/components";
import {
  menuGroupLabelClass,
  menuRowClass,
} from "@flow/features/common/ThemeLanguageMenu";
import { useRestrictedWorkspaces } from "@flow/hooks";
import { useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentUserRole, useCurrentWorkspace } from "@flow/stores";
import { Role } from "@flow/types";
import type { Workspace } from "@flow/types";
import {
  dashboardNewWorkspaceUrl,
  dashboardWorkspaceMembersUrl,
  dashboardWorkspaceSettingsUrl,
  openLinkInNewTab,
} from "@flow/utils";

import { WorkspaceAddDialog } from "./WorkspaceAddDialog";

const WorkspaceAvatar: React.FC<{ name?: string; personal?: boolean }> = ({
  name,
  personal,
}) => (
  <div
    className={`flex size-6 shrink-0 items-center justify-center bg-muted text-xs font-normal text-foreground ${personal ? "rounded-full" : "rounded-md"}`}>
    {name?.charAt(0).toUpperCase() ?? "?"}
  </div>
);

const WorkspaceMenu: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentUserRole] = useCurrentUserRole();
  const { useGetWorkspaces } = useWorkspace();
  const navigate = useNavigate();
  const { workspaces: rawWorkspaces } = useGetWorkspaces();

  const personalWorkspace = rawWorkspaces?.find(
    (workspace) => workspace.personal,
  );
  const workspaces = rawWorkspaces?.filter((rw) => !rw.personal);

  const { restrictedWorkspaceIds } = useRestrictedWorkspaces(rawWorkspaces);
  // With a dashboard configured, workspaces and their settings live there.
  const externalNewWorkspaceUrl = dashboardNewWorkspaceUrl();
  const alias = currentWorkspace?.alias;
  const externalSettingsUrl = alias
    ? dashboardWorkspaceSettingsUrl(alias)
    : undefined;
  const externalMembersUrl = alias
    ? dashboardWorkspaceMembersUrl(alias)
    : undefined;

  const canManage = currentUserRole !== Role.Reader;
  // A personal workspace has no members to manage in the Dashboard.
  const showMembers = !(externalMembersUrl && currentWorkspace?.personal);

  const [openDropdown, setOpenDropdown] = useState(false);
  const [openWorkspaceAddDialog, setOpenWorkspaceAddDialog] = useState(false);

  const handleNewWorkspace = () => {
    if (externalNewWorkspaceUrl) {
      openLinkInNewTab(externalNewWorkspaceUrl)();
      return;
    }
    setOpenWorkspaceAddDialog(true);
  };

  const handleWorkspaceChange = (workspace: Workspace) => {
    const route = window.location.pathname;
    navigate({
      to: route.replace(currentWorkspace?.id as string, workspace.id),
    });
  };

  const handleSettingsOpen = (tab: "general" | "members") => {
    const externalUrl =
      tab === "general" ? externalSettingsUrl : externalMembersUrl;
    if (externalUrl) {
      openLinkInNewTab(externalUrl)();
      return;
    }
    navigate({ to: `/workspaces/${currentWorkspace?.id}/settings/${tab}` });
  };

  const renderWorkspaceItem = (workspace: Workspace) => {
    const isRestricted = restrictedWorkspaceIds.has(workspace.id);
    return (
      <DropdownMenuItem
        key={workspace.id}
        disabled={isRestricted}
        className={`gap-2 rounded-md px-4 py-[5px] font-light ${currentWorkspace?.id === workspace.id ? "bg-accent" : ""}`}
        onClick={() => handleWorkspaceChange(workspace)}>
        <WorkspaceAvatar name={workspace.name} personal={workspace.personal} />
        <div className="flex min-w-0 flex-1 flex-col">
          <p className="truncate">{workspace.name}</p>
          {isRestricted && (
            <p className="text-[10px] text-muted-foreground">
              {t("Not available")}
            </p>
          )}
        </div>
      </DropdownMenuItem>
    );
  };

  return (
    <>
      <DropdownMenu open={openDropdown} onOpenChange={setOpenDropdown}>
        <DropdownMenuTrigger
          className={`flex w-full flex-col items-start gap-0.5 rounded-md py-0.5 pl-1 hover:bg-accent ${openDropdown ? "bg-accent" : ""}`}>
          <p className="text-xs font-light text-muted-foreground">
            {t("Current workspace:")}
          </p>
          <div className="flex w-full min-w-0 items-center gap-1">
            <WorkspaceAvatar
              name={currentWorkspace?.name}
              personal={currentWorkspace?.personal}
            />
            <p className="truncate text-sm">{currentWorkspace?.name}</p>
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="flex w-[230px] flex-col gap-0.5"
          align="start"
          side="bottom">
          <DropdownMenuLabel className={menuGroupLabelClass}>
            {t("Workspace")}
          </DropdownMenuLabel>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger className="gap-2 rounded-md px-4 py-[5px] font-light [&>svg:last-child]:text-muted-foreground">
              <WorkspaceAvatar
                name={currentWorkspace?.name}
                personal={currentWorkspace?.personal}
              />
              <p className="min-w-0 flex-1 truncate">
                {currentWorkspace?.name}
              </p>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="flex w-[230px] flex-col gap-0.5 rounded-lg">
              {personalWorkspace && (
                <>
                  <DropdownMenuLabel className={menuGroupLabelClass}>
                    {t("Personal Workspace")}
                  </DropdownMenuLabel>
                  {renderWorkspaceItem(personalWorkspace)}
                </>
              )}
              {!!workspaces?.length && (
                <>
                  <DropdownMenuLabel className={menuGroupLabelClass}>
                    {t("Team Workspaces")}
                  </DropdownMenuLabel>
                  <ScrollArea>
                    <div className="flex max-h-[40vh] flex-col gap-0.5">
                      {workspaces.map(renderWorkspaceItem)}
                    </div>
                  </ScrollArea>
                </>
              )}
              <DropdownMenuSeparator className="mx-0 my-0" />
              <DropdownMenuItem
                className={`${menuRowClass} gap-2`}
                onClick={handleNewWorkspace}>
                <PlusIcon weight="light" />
                <p className="flex-1 truncate">{t("New Workspace")}</p>
                {externalNewWorkspaceUrl && <ArrowUpRightIcon weight="light" />}
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          {canManage && (
            <DropdownMenuItem
              className={menuRowClass}
              onClick={() => handleSettingsOpen("general")}>
              <p className="flex-1 truncate">{t("Workspace Settings")}</p>
              {externalSettingsUrl && <ArrowUpRightIcon weight="light" />}
            </DropdownMenuItem>
          )}
          {canManage && showMembers && (
            <DropdownMenuItem
              className={menuRowClass}
              onClick={() => handleSettingsOpen("members")}>
              <p className="flex-1 truncate">{t("Members")}</p>
              {externalMembersUrl && <ArrowUpRightIcon weight="light" />}
            </DropdownMenuItem>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
      {openWorkspaceAddDialog && (
        <WorkspaceAddDialog
          isOpen={openWorkspaceAddDialog}
          onOpenChange={setOpenWorkspaceAddDialog}
        />
      )}
    </>
  );
};

export { WorkspaceMenu };
