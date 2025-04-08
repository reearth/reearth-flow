import { memo } from "react";

import { WorkflowTabs } from "..";

import { ActionBar, Breadcrumb, DebugActionBar, HomeMenu } from "./components";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  allowedToDeploy: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectShare: (share: boolean) => void;
  onRightPanelOpen: (content?: "version-history") => void;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowRename: (id: string, name: string) => void;
};

const TopBar: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  allowedToDeploy,
  onWorkflowDeployment,
  onProjectShare,
  onRightPanelOpen,
  onDebugRunStart,
  onDebugRunStop,
  onWorkflowClose,
  onWorkflowChange,
  onWorkflowRename,
}) => {
  return (
    <div className="flex shrink-0 justify-between gap-2 bg-secondary w-[100vw]">
      <div className="flex items-center gap-1">
        <HomeMenu
          dropdownPosition="bottom"
          dropdownAlign="end"
          dropdownAlignOffset={-140}
        />
        <div className="pr-4">
          <Breadcrumb />
        </div>
      </div>
      <div className="flex flex-1 gap-2 h-full overflow-hidden">
        <WorkflowTabs
          currentWorkflowId={currentWorkflowId}
          openWorkflows={openWorkflows}
          onWorkflowClose={onWorkflowClose}
          onWorkflowChange={onWorkflowChange}
          onWorkflowRename={onWorkflowRename}
        />
      </div>
      <div className="flex select-none items-center h-full justify-center gap-2 self-center p-1">
        <div className="border-r h-4/5" />
        <DebugActionBar
          onDebugRunStart={onDebugRunStart}
          onDebugRunStop={onDebugRunStop}
        />
        <div className="border-r h-4/5" />
        <ActionBar
          allowedToDeploy={allowedToDeploy}
          onProjectShare={onProjectShare}
          onWorkflowDeployment={onWorkflowDeployment}
          onRightPanelOpen={onRightPanelOpen}
        />
      </div>
    </div>
  );
};

export default memo(TopBar);
