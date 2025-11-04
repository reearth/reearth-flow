import { CaretDownIcon, PlusIcon } from "@phosphor-icons/react";
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
  const { workspaces: rawWorkflows } = useGetWorkspaces();

  const personalWorkspace = rawWorkflows?.find(
    (workspace) => workspace.personal,
  );
  const workspaces = rawWorkflows?.filter((rw) => !rw.personal);

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
        <DropdownMenuTrigger
          className={`group flex gap-2 overflow-auto rounded-md p-2 ${openDropdown ? "bg-background" : undefined} hover:bg-primary`}>
          <div className="relative flex w-full gap-1">
            <div className="flex w-full flex-col gap-1">
              <p className="self-start text-xs font-thin">
                {t("Current workspace:")}
              </p>
              {/* <div className="flex justify-center gap-2"> */}
              <p className="line-clamp-2 pr-1 text-sm font-light italic">
                {currentWorkspace?.name}
              </p>
            </div>
            <div className="absolute right-1 bottom-1/2 flex shrink-0 translate-1/2 items-center justify-center opacity-0 group-hover:opacity-100">
              <CaretDownIcon size="12px" weight="thin" />
            </div>
            {/* </div> */}
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="w-[224px] border"
          alignOffset={-20}
          align="center"
          side="bottom">
          <DropdownMenuGroup className="flex flex-col gap-1 overflow-auto">
            {personalWorkspace && (
              <>
                <p className="p-1 text-xs">{t("Personal Workspace")}</p>
                <DropdownMenuItem
                  key={personalWorkspace.id}
                  className={`rounded-md px-3 py-1 text-sm ${currentWorkspace?.id === personalWorkspace.id ? "bg-accent" : ""}`}
                  onClick={() => handleWorkspaceChange(personalWorkspace)}>
                  <p className="w-full truncate font-extralight">
                    {personalWorkspace.name}
                  </p>
                </DropdownMenuItem>
              </>
            )}
            <p className="p-1 text-xs">{t("Team Workspaces")}</p>
            <ScrollArea>
              <div className="flex max-h-[40vh] flex-col gap-1 overflow-auto">
                {workspaces?.map((workspace) => (
                  <DropdownMenuItem
                    key={workspace.id}
                    className={`mx-1 rounded-md px-2 py-[2px] text-sm ${currentWorkspace?.id === workspace.id ? "bg-accent" : ""}`}
                    onClick={() => handleWorkspaceChange(workspace)}>
                    <p className="w-full truncate font-extralight">
                      {workspace.name}
                    </p>
                  </DropdownMenuItem>
                ))}
              </div>
            </ScrollArea>
          </DropdownMenuGroup>
          <div className="-mx-2 mt-1 border-t pb-1" />
          <div
            className="flex w-full cursor-pointer justify-center gap-2 rounded-md py-2 hover:bg-primary"
            onClick={() => {
              setOpenWorkspaceAddDialog(true);
              setOpenDropdown(false);
            }}>
            <PlusIcon weight="thin" />
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
