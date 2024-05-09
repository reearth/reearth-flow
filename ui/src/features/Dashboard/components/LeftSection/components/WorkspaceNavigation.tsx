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

type Props = {
  className?: string;
};

const WorkspaceNavigation: React.FC<Props> = ({ className }) => {
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const [, setCurrentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "/workspace/$workspaceId" });

  const handleWorkspaceChange = (workspace: Workspace) => {
    setCurrentProject(undefined);
    setCurrentWorkspace(workspace);
    navigate({ to: `/workspace/${workspace.id}` });
  };

  return (
    <div className={`flex flex-col gap-4  ${className}`}>
      <DropdownMenu>
        <DropdownMenuTrigger className="flex items-center px-2 -mx-2 rounded-md hover:bg-zinc-700/50">
          <p className="text-md uppercase font-thin truncate">{currentWorkspace?.name}</p>
          <ChevronDown className="ml-2" size="12px" />
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
        </DropdownMenuContent>
      </DropdownMenu>
      {/* <Tooltip>
        <TooltipTrigger className="self-end">
          <p className="flex font-thin items-center bg-zinc-800/80 px-1 rounded-md">
            {currentWorkspace?.members?.length} <PersonIcon />
          </p>
        </TooltipTrigger>
        <TooltipContent className="flex flex-col gap-1 max-h-[160px]" side="bottom" align="start">
          <div className="overflow-scroll">
            {currentWorkspace?.members?.map(member => <p key={member.id}>{member.name}</p>)}
          </div>
        </TooltipContent>
      </Tooltip> */}
    </div>
  );
};

export { WorkspaceNavigation };
