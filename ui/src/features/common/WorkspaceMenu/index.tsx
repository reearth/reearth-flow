import { CaretRight, Plus } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
  ScrollArea,
} from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Workspace } from "@flow/types";

import { WorkspaceAddDialog } from "./WorkspaceAddDialog";

const WorkspaceMenu: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { useGetWorkspaces } = useWorkspace();
  const navigate = useNavigate();
  const { workspaces } = useGetWorkspaces();

  const [openDropdown, setOpenDropdown] = useState(false);
  const [openWorkspaceAddDialog, setOpenWorkspaceAddDialog] = useState(false);

  const handleWorkspaceChange = (workspace: Workspace) => {
    const route = window.location.pathname;
    navigate({
      to: route.replace(currentWorkspace?.id as string, workspace.id),
    });
  };

  return (
    <>
      <DropdownMenu
        open={openDropdown}
        onOpenChange={(o) => setOpenDropdown(o)}>
        <DropdownMenuTrigger className="-mx-2 flex max-w-[213px] items-center gap-2 overflow-auto rounded-md bg-background p-1 hover:bg-primary">
          <p className="line-clamp-2 flex-1 text-sm font-extralight">
            {currentWorkspace?.personal
              ? t("Personal workspace")
              : currentWorkspace?.name}
          </p>
          <div className="shrink-0">
            <CaretRight size="12px" weight="thin" />
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="min-w-[150px] max-w-[300px] border"
          sideOffset={5}
          alignOffset={-50}
          align="end"
          side="right">
          <ScrollArea>
            <DropdownMenuGroup className="flex max-h-[50vh] flex-col gap-1 overflow-auto">
              {workspaces?.map((workspace) => (
                <DropdownMenuItem
                  key={workspace.id}
                  className={`rounded-md px-4 py-1 ${currentWorkspace?.id === workspace.id ? "bg-accent" : ""}`}
                  onClick={() => handleWorkspaceChange(workspace)}>
                  <p className="w-full truncate text-center font-extralight">
                    {workspace.name}
                    {workspace.personal && (
                      <span className="font-normal">{t(" (Personal)")}</span>
                    )}
                  </p>
                </DropdownMenuItem>
              ))}
            </DropdownMenuGroup>
          </ScrollArea>
          <div className="-mx-2 mt-1 border-t pb-1" />
          <div
            className="flex w-full cursor-pointer justify-center gap-2 rounded-md py-2 hover:bg-primary"
            onClick={() => {
              setOpenWorkspaceAddDialog(true);
              setOpenDropdown(false);
            }}>
            <Plus weight="thin" />
            <p className="text-xs dark:font-light">{t("New Workspace")}</p>
          </div>
        </DropdownMenuContent>
      </DropdownMenu>
      {openWorkspaceAddDialog && (
        <WorkspaceAddDialog
          isOpen={openWorkspaceAddDialog}
          onOpenChange={(o) => setOpenWorkspaceAddDialog(o)}
        />
      )}
    </>
  );
};

export { WorkspaceMenu };
