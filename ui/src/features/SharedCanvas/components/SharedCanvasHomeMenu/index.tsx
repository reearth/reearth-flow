import { ChalkboardTeacherIcon, DotsThreeIcon } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";
import { memo, useState } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  FlowLogo,
} from "@flow/components";
import { WorkflowsDropdown } from "@flow/features/Editor/components/OverlayUI/components/Homebar/components";
import { useT } from "@flow/lib/i18n";
import type { Project, Workspace } from "@flow/types";

import { SharedCanvasDialogOptions } from "../../types";
import SharedCanvasWorkflowVariables from "../SharedCanvasWorkflowVariables";

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
  const t = useT();
  const [showDialog, setShowDialog] =
    useState<SharedCanvasDialogOptions>(undefined);

  const handleDialogOpen = (dialog: SharedCanvasDialogOptions) =>
    setShowDialog(dialog);
  const handleDialogClose = () => setShowDialog(undefined);

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
        <DropdownMenu>
          <DropdownMenuTrigger
            asChild
            className="h-6 w-8 shrink-0 cursor-pointer rounded p-0.5 hover:bg-primary">
            <DotsThreeIcon weight="light" />
          </DropdownMenuTrigger>
          <DropdownMenuContent
            side="bottom"
            align="start"
            sideOffset={10}
            alignOffset={-6}>
            <DropdownMenuItem
              onClick={() => handleDialogOpen("workflowVariables")}>
              <ChalkboardTeacherIcon weight="thin" size={18} />
              <p>{t("Workflow Variables")}</p>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <WorkflowsDropdown
          openWorkflows={openWorkflows}
          currentWorkflowId={currentWorkflowId}
          onWorkflowChange={onWorkflowChange}
          onWorkflowClose={onWorkflowClose}
        />
        {showDialog === "workflowVariables" && (
          <SharedCanvasWorkflowVariables
            project={project}
            isOpen={showDialog === "workflowVariables"}
            onOpenChange={(open) =>
              !open
                ? handleDialogClose()
                : handleDialogOpen("workflowVariables")
            }
            onCancel={handleDialogClose}
          />
        )}
      </div>
    </div>
  );
};

export default memo(SharedCanvasHomeMenu);
