import { useNavigate } from "@tanstack/react-router";
import { memo } from "react";

import { FlowLogo } from "@flow/components";
import { WorkflowsDropdown } from "@flow/features/Editor/components/OverlayUI/components/Homebar/components";
import type { Project, Workspace } from "@flow/types";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  project?: Project;
  isMainWorkflow: boolean;
  workspaces?: Workspace[] | undefined;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
};

const SharedCanvasHomeMenu: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  project,
  isMainWorkflow,
  onWorkflowClose,
  onWorkflowChange,
}) => {
  const navigate = useNavigate();

  return (
    <div
      className={`rounded-xl border bg-secondary/70 px-2 py-1 shadow-md shadow-secondary backdrop-blur-xs ${isMainWorkflow ? "border-primary" : "border-node-subworkflow"}`}>
      <div className="flex h-[42px] min-w-[250px] items-center gap-4 self-start">
        <div className="ml-1" onClick={() => navigate({ to: "/" })}>
          <FlowLogo className="size-7 cursor-pointer transition-all" />
        </div>
        <div className="flex-1">
          <p className="min-w-[100px] truncate text-center text-sm font-bold transition-all duration-500">
            {project?.name}
          </p>
        </div>
        <div className="w-6" />
      </div>

      <div className="flex h-[30px] items-center gap-1">
        <WorkflowsDropdown
          openWorkflows={openWorkflows}
          currentWorkflowId={currentWorkflowId}
          onWorkflowChange={onWorkflowChange}
          onWorkflowClose={onWorkflowClose}
        />
      </div>
    </div>
  );
};

export default memo(SharedCanvasHomeMenu);
