import { ChevronRightIcon, PersonIcon } from "@radix-ui/react-icons";
import { ChevronDown } from "lucide-react";
import { useContext } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import { DialogContext } from "@flow/features/Dialog";
import { cn } from "@flow/lib/utils";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import { Workspace } from "@flow/types";

const WorkspaceNavigation: React.FC = () => {
  const dialogContext = useContext(DialogContext);

  const workspaces = dialogContext?.workspaces;
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const [, setCurrentProject] = useCurrentProject();

  const handleWorkspaceChange = (workspace: Workspace) => {
    setCurrentProject(undefined);
    setCurrentWorkspace(workspace);
  };

  return (
    <div className="flex gap-4">
      <DropdownMenu>
        <DropdownMenuTrigger className="flex items-center px-2 -mx-2 rounded-md hover:bg-zinc-800 first:[data-state=open]:bg-zinc-800">
          <p className="text-xl uppercase font-bold truncate max-w-[350px]">
            {currentWorkspace?.name}
          </p>
          <ChevronDown className="ml-2" size="12px" />
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="max-w-[300px] min-w-[150px] bg-zinc-900 border-none"
          align="start">
          {/* <DropdownMenuLabel>Workspaces</DropdownMenuLabel> */}
          {/* <div className="bg-zinc-800 h-[1px]" /> */}
          <DropdownMenuGroup className="max-h-[300px] overflow-scroll">
            {workspaces?.map(workspace => (
              <DropdownMenuItem
                key={workspace.id}
                className={cn(
                  "rounded-md mr-1 mt-1 mb-1 pl-0",
                  currentWorkspace?.id === workspace.id ? "bg-zinc-800/50" : undefined,
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
      <Tooltip>
        <TooltipTrigger className="self-center">
          <p className="flex font-thin items-center bg-zinc-800/80 px-1 rounded-md">
            {currentWorkspace?.members?.length} <PersonIcon />
          </p>
        </TooltipTrigger>
        <TooltipContent className="flex flex-col gap-1 max-h-[160px]" side="bottom" align="start">
          <div className="overflow-scroll">
            {currentWorkspace?.members?.map(member => <p key={member.id}>{member.name}</p>)}
          </div>
        </TooltipContent>
      </Tooltip>
    </div>
  );
};

export { WorkspaceNavigation };
