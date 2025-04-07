import { useNavigate, useParams } from "@tanstack/react-router";
import { memo, useCallback } from "react";

import { FlowLogo } from "@flow/components";

import { WorkflowTabs } from "..";

import { ActionBar, Breadcrumb } from "./components";

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
  const navigate = useNavigate();
  const { workspaceId } = useParams({ strict: false });

  const handleNavigationToDashboard = useCallback(() => {
    navigate({ to: `/workspaces/${workspaceId}/projects` });
  }, [workspaceId, navigate]);
  return (
    <div className="flex shrink-0 justify-between gap-2 bg-secondary w-[100vw]">
      <div className="flex gap-2 h-full">
        <div
          className="flex items-center gap-6 px-4"
          onClick={handleNavigationToDashboard}>
          <div className="box-content">
            <FlowLogo className="size-6 transition-all hover:text-[#46ce7c] cursor-pointer" />
          </div>
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
        <ActionBar
          allowedToDeploy={allowedToDeploy}
          onProjectShare={onProjectShare}
          onWorkflowDeployment={onWorkflowDeployment}
          onDebugRunStart={onDebugRunStart}
          onDebugRunStop={onDebugRunStop}
          onRightPanelOpen={onRightPanelOpen}
        />
      </div>
    </div>
  );
};

export default memo(TopBar);
