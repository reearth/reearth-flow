import { ChevronRightIcon } from "@radix-ui/react-icons";
import { useNavigate } from "@tanstack/react-router";
import { ChevronDown } from "lucide-react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { cn } from "@flow/lib/utils";
import { workspaces } from "@flow/mock_data/workspaceData";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import { Workspace } from "@flow/types";

const WorkspaceNavigation: React.FC = () => {
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const [, setCurrentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "/workspace/$workspaceId" });

  const handleWorkspaceChange = (workspace: Workspace) => {
    setCurrentProject(undefined);
    setCurrentWorkspace(workspace);
    navigate({ to: `/workspace/${workspace.id}` });
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="flex items-center px-2 -mx-2 rounded-md max-w-[30vw] hover:bg-zinc-700/50">
        <p className="text-md uppercase font-thin truncate">{currentWorkspace?.name}</p>
        <div className="ml-2">
          <ChevronDown size="12px" />
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        className="max-w-[300px] min-w-[150px] bg-zinc-800 border"
        sideOffset={10}
        align="center">
        {/* <DropdownMenuLabel>Workspaces</DropdownMenuLabel> */}
        {/* <div className="bg-zinc-800 h-[1px]" /> */}
        <DropdownMenuGroup className="max-h-[300px] overflow-auto">
          {workspaces?.map(workspace => (
            <DropdownMenuItem
              key={workspace.id}
              className={cn(
                "rounded-md mr-1 mt-1 mb-1 pl-0",
                currentWorkspace?.id === workspace.id ? "bg-zinc-700/50" : undefined,
              )}
              onClick={() => handleWorkspaceChange(workspace)}>
              <div className="flex items-center justify-center w-[15px] h-[15px] mr-1">
                {currentWorkspace?.id === workspace.id && (
                  <ChevronRightIcon className="text-zinc-500" />
                )}
              </div>
              <span className="truncate text-xs">{workspace.name}</span>
            </DropdownMenuItem>
          ))}
        </DropdownMenuGroup>
        {/* <DropdownMenuLabel className="flex">
            <Button className="flex flex-1 gap-2" variant="outline" size="sm">
              <PlusCircledIcon />
              <p>New workspace</p>
            </Button>
          </DropdownMenuLabel> */}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

export { WorkspaceNavigation };
