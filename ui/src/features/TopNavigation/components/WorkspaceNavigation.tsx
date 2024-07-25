import { CaretDown } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { cn } from "@flow/lib/utils";
import { useCurrentWorkspace } from "@flow/stores";
import { Workspace } from "@flow/types";

const WorkspaceNavigation: React.FC = () => {
  const [currentWorkspace] = useCurrentWorkspace();
  const { useGetWorkspaces } = useWorkspace();
  const navigate = useNavigate();
  const { workspaces } = useGetWorkspaces();

  const handleWorkspaceChange = (workspace: Workspace) => {
    const route = window.location.pathname;
    navigate({ to: route.replace(currentWorkspace?.id as string, workspace.id) });
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="-mx-2 flex max-w-[30vw] items-center rounded-md px-2 py-1 hover:bg-zinc-700/50">
        <p className="truncate text-lg font-thin">{currentWorkspace?.name}</p>
        <div className="ml-2">
          <CaretDown size="12px" />
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="min-w-[150px] max-w-[300px] border bg-zinc-800"
        sideOffset={5}
        align="center">
        <DropdownMenuGroup className="max-h-[300px] overflow-auto">
          {workspaces?.map(workspace => (
            <DropdownMenuItem
              key={workspace.id}
              className={cn(
                "rounded-md my-1",
                currentWorkspace?.id === workspace.id ? "bg-zinc-700/50" : undefined,
              )}
              onClick={() => handleWorkspaceChange(workspace)}>
              <p className="w-full truncate text-center font-thin">{workspace.name}</p>
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { WorkspaceNavigation };
