import { FileIcon, SlidersHorizontalIcon } from "@phosphor-icons/react";
import { memo } from "react";

import { ButtonWithTooltip, EcosystemNavigator } from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import { useT } from "@flow/lib/i18n";
import type { OpenWorkflow } from "@flow/lib/yjs/useWorkflowTabs";
import type { AwarenessUser } from "@flow/types";

import {
  CollaborationActionBar,
  HomeMenu,
  ProjectMenu,
  WorkflowVariablesDialog,
  WorkflowsDropdown,
} from "./components";
import useHooks from "./hooks";

type Props = {
  isMainWorkflow: boolean;
  self: AwarenessUser;
  users: Record<string, AwarenessUser>;
  spotlightUserClientId: number | null;
  currentWorkflowId: string;
  openWorkflows: OpenWorkflow[];
  onSpotlightUserSelect: (clientId: number) => void;
  onSpotlightUserDeselect: () => void;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onUserFocusedElement?: (isOpen: boolean) => void;
};

const Homebar: React.FC<Props> = ({
  isMainWorkflow,
  self,
  users,
  spotlightUserClientId,
  currentWorkflowId,
  openWorkflows,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
  onWorkflowChange,
  onWorkflowClose,
  onUserFocusedElement,
}) => {
  const t = useT();

  const {
    showDialog,
    currentProject,
    currentWorkflowVariables,
    handleWorkflowVariableAdd,
    handleWorkflowVariableChange,
    handleWorkflowVariablesBatchUpdate,
    handleWorkflowVariableDelete,
    handleWorkflowVariablesBatchDelete,
    handleDialogOpen,
    handleDialogClose,
  } = useHooks({
    onUserFocusedElement,
  });

  return (
    <div
      className={`flex min-w-[300px] flex-col gap-2 rounded-xl border bg-secondary/70 p-2 shadow-md shadow-[black]/10 backdrop-blur-xs dark:shadow-secondary ${isMainWorkflow ? "border-border dark:border-primary" : "border-node-subworkflow"}`}>
      <div className="flex items-center justify-between gap-4">
        <div className="flex min-w-0 items-center gap-2">
          <HomeMenu />
          <EcosystemNavigator iconSize={18} />
          <ProjectMenu />
        </div>
        <CollaborationActionBar
          self={self}
          users={users}
          showDialog={showDialog}
          spotlightUserClientId={spotlightUserClientId}
          onDialogOpen={handleDialogOpen}
          onDialogClose={handleDialogClose}
          onSpotlightUserSelect={onSpotlightUserSelect}
          onSpotlightUserDeselect={onSpotlightUserDeselect}
        />
      </div>
      <div className="flex items-center justify-between gap-1">
        <WorkflowsDropdown
          openWorkflows={openWorkflows}
          currentWorkflowId={currentWorkflowId}
          onWorkflowChange={onWorkflowChange}
          onWorkflowClose={onWorkflowClose}
        />
        <div className="flex items-center gap-1">
          <ButtonWithTooltip
            className="h-6"
            variant="ghost"
            tooltipText={t("Workflow Variables")}
            onClick={() => handleDialogOpen("workflowVariables")}>
            <SlidersHorizontalIcon weight="light" size={16} />
          </ButtonWithTooltip>
          <ButtonWithTooltip
            className="h-6"
            variant="ghost"
            tooltipText={t("Workspace Assets")}
            onClick={() => handleDialogOpen("assets")}>
            <FileIcon weight="light" size={16} />
          </ButtonWithTooltip>
        </div>
      </div>
      {showDialog === "workflowVariables" && (
        <WorkflowVariablesDialog
          currentWorkflowVariables={currentWorkflowVariables}
          projectId={currentProject?.id}
          users={users}
          onClose={handleDialogClose}
          onAdd={handleWorkflowVariableAdd}
          onChange={handleWorkflowVariableChange}
          onDelete={handleWorkflowVariableDelete}
          onDeleteBatch={handleWorkflowVariablesBatchDelete}
          onBatchUpdate={handleWorkflowVariablesBatchUpdate}
        />
      )}
      {showDialog === "assets" && (
        <AssetsDialog onDialogClose={handleDialogClose} />
      )}
    </div>
  );
};

export default memo(Homebar);
