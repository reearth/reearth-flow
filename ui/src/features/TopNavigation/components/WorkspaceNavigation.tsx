import { useNavigate } from "@tanstack/react-router";
import { ChevronDown } from "lucide-react";

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
      <DropdownMenuTrigger className="flex items-center py-1 px-2 -mx-2 rounded-md max-w-[30vw] hover:bg-zinc-700/50">
        <p className="text-lg font-thin truncate">{currentWorkspace?.name}</p>
        <div className="ml-2">
          <ChevronDown size="12px" />
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="max-w-[300px] min-w-[150px] bg-zinc-800 border"
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
              <p className="truncate w-full text-center font-thin">{workspace.name}</p>
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { WorkspaceNavigation };
