import { CaretDown, Plus } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace, useDialogType } from "@flow/stores";
import { Workspace } from "@flow/types";

const WorkspaceNavigation: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { useGetWorkspaces } = useWorkspace();
  const navigate = useNavigate();
  const { workspaces } = useGetWorkspaces();
  const [, setDialogType] = useDialogType();

  const handleWorkspaceChange = (workspace: Workspace) => {
    const route = window.location.pathname;
    navigate({
      to: route.replace(currentWorkspace?.id as string, workspace.id),
    });
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="-mx-2 flex max-w-[30vw] items-center rounded-md px-2 py-1 hover:bg-background">
        <p className="truncate text-lg font-thin">{currentWorkspace?.name}</p>
        <div className="ml-2">
          <CaretDown size="12px" />
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="min-w-[150px] max-w-[300px] border"
        sideOffset={5}
        align="center"
      >
        <DropdownMenuGroup className="flex max-h-[200px] flex-col gap-1 overflow-auto">
          {workspaces?.map((workspace) => (
            <DropdownMenuItem
              key={workspace.id}
              className={`rounded-md px-4 py-1 ${currentWorkspace?.id === workspace.id ? "bg-accent" : ""}`}
              onClick={() => handleWorkspaceChange(workspace)}
            >
              <p className="w-full truncate text-center font-extralight">
                {workspace.name}
              </p>
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
        <div className="-mx-2 mt-1 border-t pb-1" />
        <div
          className="flex w-full cursor-pointer justify-center gap-2 rounded-md py-2 hover:bg-primary"
          onClick={() => setDialogType("add-workspace")}
        >
          <Plus weight="thin" />
          <p className="text-xs font-light">{t("New Workspace")}</p>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { WorkspaceNavigation };
