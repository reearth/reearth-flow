import {
  ChalkboardTeacherIcon,
  FileIcon,
  GearFineIcon,
} from "@phosphor-icons/react";
import { memo } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import { useT } from "@flow/lib/i18n";
import { AwarenessUser } from "@flow/types";

import {
  Breadcrumb,
  CollaborationActionBar,
  HomeMenu,
  ProjectVariableDialog,
} from "../../../TopBar/components";

import { WorkflowsDropdown } from "./components";
import useHooks from "./hooks";

type Props = {
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
    currentProjectVariables,
    handleProjectVariableAdd,
    handleProjectVariableChange,
    handleProjectVariablesBatchUpdate,
    handleProjectVariableDelete,
    handleProjectVariablesBatchDelete,
    handleDialogOpen,
    handleDialogClose,
  } = useHooks();

  return (
    <div className="rounded-xl border border-primary bg-secondary/70 px-2 py-1 shadow-md shadow-secondary backdrop-blur-xs">
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
        <DropdownMenu>
          <DropdownMenuTrigger
            asChild
            className="h-6 w-8 shrink-0 cursor-pointer rounded p-0.5 hover:bg-primary">
            <GearFineIcon weight="thin" />
          </DropdownMenuTrigger>
          <DropdownMenuContent
            side="bottom"
            align="start"
            sideOffset={10}
            alignOffset={-6}>
            <DropdownMenuItem
              onClick={() => handleDialogOpen("projectVariables")}>
              <ChalkboardTeacherIcon weight="thin" size={18} />
              <p>{t("Project Variables")}</p>
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleDialogOpen("assets")}>
              <FileIcon weight="thin" size={18} />
              <p>{t("Workspace Assets")}</p>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <WorkflowsDropdown
          openWorkflows={openWorkflows}
          currentWorkflowId={currentWorkflowId}
          onWorkflowChange={onWorkflowChange}
          onWorkflowClose={onWorkflowClose}
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

export default memo(Homebar);
