import { useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

import { useNewWorkspace } from "./useNewWorkspace";
import { WorkspaceAvatar, WorkspaceMenuItems } from "./WorkspaceMenuItems";

const WorkspaceMenu: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const [openDropdown, setOpenDropdown] = useState(false);
  const { handleNewWorkspace, workspaceAddDialog } = useNewWorkspace();

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
          <WorkspaceMenuItems onNewWorkspace={handleNewWorkspace} />
        </DropdownMenuContent>
      </DropdownMenu>
      {workspaceAddDialog}
    </>
  );
};

export { WorkspaceAvatar, WorkspaceMenu, WorkspaceMenuItems };
