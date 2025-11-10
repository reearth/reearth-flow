import { memo } from "react";

import { WorkflowsDropdown } from "@flow/features/Editor/components/OverlayUI/components/Homebar/components";
import type { Workspace } from "@flow/types";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  workspaces?: Workspace[] | undefined;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
};

const VersionCanvasHomeMenu: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  onWorkflowClose,
  onWorkflowChange,
}) => {
  return (
    <div className="flex h-[30px] min-w-16 items-center gap-1">
      <WorkflowsDropdown
        openWorkflows={openWorkflows}
        currentWorkflowId={currentWorkflowId}
        onWorkflowChange={onWorkflowChange}
        onWorkflowClose={onWorkflowClose}
      />
    </div>
  );
};

export default memo(VersionCanvasHomeMenu);
