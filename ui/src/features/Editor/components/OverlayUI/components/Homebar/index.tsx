import { FileIcon, ListPlusIcon } from "@phosphor-icons/react";
import { memo } from "react";

import { ButtonWithTooltip } from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import { useT } from "@flow/lib/i18n";
import { AwarenessUser } from "@flow/types";

import {
  Breadcrumb,
  CollaborationActionBar,
  HomeMenu,
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
  openWorkflows: {
    id: string;
    name: string;
  }[];
  onSpotlightUserSelect: (clientId: number) => void;
  onSpotlightUserDeselect: () => void;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
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
  } = useHooks();

  return (
    <div
      className={`rounded-xl border bg-secondary/70 px-2 py-1 shadow-md shadow-[black]/10 backdrop-blur-xs dark:shadow-secondary ${isMainWorkflow ? "border-border dark:border-primary" : "border-node-subworkflow"}`}>
      <div className="flex h-[42px] min-w-[250px] items-center gap-4 self-start">
        <HomeMenu
          dropdownPosition="bottom"
          dropdownAlign="end"
          dropdownAlignOffset={-180}
        />
        <div className="flex-1">
          <Breadcrumb />
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
      <div className="flex h-[30px] items-center gap-1">
        <WorkflowsDropdown
          openWorkflows={openWorkflows}
          currentWorkflowId={currentWorkflowId}
          onWorkflowChange={onWorkflowChange}
          onWorkflowClose={onWorkflowClose}
        />
        <ButtonWithTooltip
          className="h-6"
          variant="ghost"
          tooltipText={t("Workflow Variables")}
          onClick={() => handleDialogOpen("workflowVariables")}>
          <ListPlusIcon weight="light" size={16} />
        </ButtonWithTooltip>
        <ButtonWithTooltip
          className="h-6"
          variant="ghost"
          tooltipText={t("Workspace Assets")}
          onClick={() => handleDialogOpen("assets")}>
          <FileIcon weight="thin" size={16} />
        </ButtonWithTooltip>
      </div>
      {showDialog === "workflowVariables" && (
        <WorkflowVariablesDialog
          currentWorkflowVariables={currentWorkflowVariables}
          projectId={currentProject?.id}
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
