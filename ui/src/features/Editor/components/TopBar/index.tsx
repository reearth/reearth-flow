import { ArchiveIcon, ChalkboardTeacherIcon } from "@phosphor-icons/react";
import { memo } from "react";
import { Doc } from "yjs";

import { IconButton } from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import type { AwarenessUser, Project } from "@flow/types";

import { WorkflowTabs } from "..";

import {
  ActionBar,
  Breadcrumb,
  CollaborationActionBar,
  DebugActionBar,
  HomeMenu,
  ProjectVariableDialog,
} from "./components";
import useHooks from "./hooks";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  project?: Project;
  yDoc: Doc | null;
  allowedToDeploy: boolean;
  isSaving: boolean;
  self: AwarenessUser;
  users: Record<string, AwarenessUser>;
  spotlightUserClientId: number | null;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectExport: () => void;
  onProjectShare: (share: boolean) => void;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onProjectSnapshotSave: () => Promise<void>;
  onSpotlightUserSelect: (clientId: number) => void;
  onSpotlightUserDeselect: () => void;
};

const TopBar: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  project,
  yDoc,
  allowedToDeploy,
  isSaving,
  self,
  users,
  spotlightUserClientId,
  onWorkflowDeployment,
  onProjectExport,
  onProjectShare,
  onDebugRunStart,
  onDebugRunStop,
  onWorkflowClose,
  onWorkflowChange,
  onProjectSnapshotSave,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
}) => {
  const t = useT();
  const {
    isMainWorkflow,
    showDialog,
    currentProjectVariables,
    handleProjectVariableAdd,
    handleProjectVariableChange,
    handleProjectVariablesBatchUpdate,
    handleProjectVariableDelete,
    handleProjectVariablesBatchDelete,
    handleDialogOpen,
    handleDialogClose,
  } = useHooks({ openWorkflows, currentWorkflowId });
  const [currentProject] = useCurrentProject();

  return (
    <div className="flex h-[50px] w-[100vw] shrink-0 justify-between bg-secondary">
      <div
        className={`flex items-center gap-1 border-b pr-2 pl-1 ${!isMainWorkflow ? "border-node-subworkflow" : ""}`}>
        <HomeMenu
          dropdownPosition="bottom"
          dropdownAlign="end"
          dropdownAlignOffset={-170}
        />
        <div className="pr-2">
          <Breadcrumb />
        </div>
        <div className="flex items-center gap-2 rounded-md">
          <IconButton
            className="h-8"
            variant="outline"
            tooltipText={t("Project Variables")}
            icon={<ChalkboardTeacherIcon weight="thin" size={18} />}
            onClick={() => handleDialogOpen("projectVariables")}
          />
          <IconButton
            variant="outline"
            tooltipText={t("Assets")}
            icon={<ArchiveIcon weight="thin" size={18} />}
            onClick={() => handleDialogOpen("assets")}
          />
        </div>
      </div>
      <div className="flex h-full flex-1 gap-2 overflow-hidden">
        <WorkflowTabs
          currentWorkflowId={currentWorkflowId}
          openWorkflows={openWorkflows}
          onWorkflowClose={onWorkflowClose}
          onWorkflowChange={onWorkflowChange}
        />
      </div>
      <div
        className={`flex h-full items-center justify-center gap-2 self-center border-b px-1 select-none ${!isMainWorkflow ? "border-node-subworkflow" : ""}`}>
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
        <div className="h-4/5 border-r" />
        <DebugActionBar
          onDebugRunStart={onDebugRunStart}
          onDebugRunStop={onDebugRunStop}
        />
        <div className="h-4/5 border-r" />
        <ActionBar
          project={project}
          yDoc={yDoc}
          allowedToDeploy={allowedToDeploy}
          showDialog={showDialog}
          isSaving={isSaving}
          onProjectShare={onProjectShare}
          onProjectExport={onProjectExport}
          onWorkflowDeployment={onWorkflowDeployment}
          onDialogOpen={handleDialogOpen}
          onDialogClose={handleDialogClose}
          onProjectSnapshotSave={onProjectSnapshotSave}
        />
      </div>
      {showDialog === "projectVariables" && (
        <ProjectVariableDialog
          currentProjectVariables={currentProjectVariables}
          projectId={currentProject?.id}
          onClose={handleDialogClose}
          onAdd={handleProjectVariableAdd}
          onChange={handleProjectVariableChange}
          onDelete={handleProjectVariableDelete}
          onDeleteBatch={handleProjectVariablesBatchDelete}
          onBatchUpdate={handleProjectVariablesBatchUpdate}
        />
      )}
      {showDialog === "assets" && (
        <AssetsDialog onDialogClose={handleDialogClose} />
      )}
    </div>
  );
};

export default memo(TopBar);
