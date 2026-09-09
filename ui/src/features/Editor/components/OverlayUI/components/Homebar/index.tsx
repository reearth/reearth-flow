import { FileIcon, ListPlusIcon } from "@phosphor-icons/react";
import { memo } from "react";

import { ButtonWithTooltip } from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import type { UserActivity } from "@flow/features/Editor/useUserActivities";
import { useT } from "@flow/lib/i18n";
import type { SpotlightFollow } from "@flow/lib/yjs";
import { AwarenessDialog, AwarenessUser } from "@flow/types";

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
  spotlightFollow: SpotlightFollow;
  userActivities: Record<string, UserActivity>;
  onDialogAwareness?: (
    dialog: AwarenessDialog | null,
    ownedDialogs: readonly AwarenessDialog[],
  ) => void;
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
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
  spotlightFollow,
  userActivities,
  onDialogAwareness,
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
    onDialogAwareness,
    isFollowing: spotlightFollow.isFollowing,
    followDialog: spotlightFollow.dialog,
  });

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
          userActivities={userActivities}
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
      {/* Editing surfaces hand control back to the follower on interaction,
          matching the canvas' click-to-unfollow behaviour. Portaled dialog
          content still bubbles through the React tree. Read-only browse
          surfaces (assets, version history) deliberately do not — clicking
          around in them is looking, not taking over. */}
      <div className="contents" onPointerDownCapture={onSpotlightUserDeselect}>
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
      </div>
      {showDialog === "assets" && (
        <AssetsDialog onDialogClose={handleDialogClose} />
      )}
    </div>
  );
};

export default memo(Homebar);
