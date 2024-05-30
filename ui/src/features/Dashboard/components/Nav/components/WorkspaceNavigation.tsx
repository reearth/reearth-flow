import { ChevronRightIcon, PlusIcon } from "@radix-ui/react-icons";
import { useNavigate } from "@tanstack/react-router";
import { ChevronDown } from "lucide-react";

import {
  ButtonWithTooltip,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useGetWorkspaceQuery } from "@flow/lib/api";
import { Workspace } from "@flow/lib/gql";
import { cn } from "@flow/lib/utils";
import { useT } from "@flow/providers";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

type Props = {
  className?: string;
};

const WorkspaceNavigation: React.FC<Props> = ({ className }) => {
  const t = useT();
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const [, setCurrentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "/workspace/$workspaceId" });

  // TODO: This fails with proper workspaces
  const handleWorkspaceChange = (workspace: Workspace) => {
    setCurrentProject(undefined);
    setCurrentWorkspace(workspace);
    navigate({ to: `/workspace/${workspace.id}` });
  };

  const { data } = useGetWorkspaceQuery();

  const workspaces = data?.me?.workspaces;

  return (
    <div className={`flex justify-center gap-4 ${className}`}>
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
                // TODO: Fix TS error
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
      <ButtonWithTooltip
        className="bg-zinc-800 hover:bg-zinc-700"
        variant="outline"
        size="icon"
        tooltipText={t("Create new workspace")}>
        <PlusIcon />
      </ButtonWithTooltip>
    </div>
  );
};

export { WorkspaceNavigation };
