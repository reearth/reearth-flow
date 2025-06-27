import { ChalkboardTeacherIcon, HardDriveIcon } from "@phosphor-icons/react";
import { memo, useState, useCallback, useMemo } from "react";
import { Doc } from "yjs";

import { IconButton } from "@flow/components";
import { useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import {
  ProjectVariable as ProjectVariableType,
  Project,
  AnyProjectVariable,
} from "@flow/types";

import { WorkflowTabs } from "..";

import {
  ActionBar,
  Breadcrumb,
  DebugActionBar,
  HomeMenu,
  ProjectVariableDialog,
} from "./components";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  project?: Project;
  yDoc: Doc | null;
  allowedToDeploy: boolean;
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
};

const TopBar: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  project,
  yDoc,
  allowedToDeploy,
  onWorkflowDeployment,
  onProjectExport,
  onProjectShare,
  onDebugRunStart,
  onDebugRunStop,
  onWorkflowClose,
  onWorkflowChange,
}) => {
  const t = useT();
  const [showProjectVarsDialog, setShowProjectVarsDialog] = useState(false);
  const [currentProject] = useCurrentProject();

  const {
    useGetProjectVariables,
    createProjectVariable,
    updateMultipleProjectVariables,
    deleteProjectVariable,
    deleteProjectVariables,
  } = useProjectVariables();

  const { projectVariables } = useGetProjectVariables(currentProject?.id);

  const currentProjectVariables = useMemo(
    () => projectVariables ?? [],
    [projectVariables],
  );

  const handleProjectVariableAdd = useCallback(
    async (projectVariable: ProjectVariableType) => {
      if (!currentProject) return;

      await createProjectVariable(
        currentProject.id,
        projectVariable.name,
        projectVariable.defaultValue,
        projectVariable.type,
        projectVariable.required,
        projectVariable.public,
        currentProjectVariables.length,
        projectVariable.config,
      );
    },
    [currentProject, createProjectVariable, currentProjectVariables.length],
  );

  const handleProjectVariableChange = useCallback(
    async (projectVariable: ProjectVariableType) => {
      if (!currentProject) return;

      await updateMultipleProjectVariables({
        projectId: currentProject.id,
        updates: [
          {
            paramId: projectVariable.id,
            name: projectVariable.name,
            defaultValue: projectVariable.defaultValue,
            type: projectVariable.type,
            required: projectVariable.required,
            publicValue: projectVariable.public,
            config: projectVariable.config,
          },
        ],
      });
    },
    [updateMultipleProjectVariables, currentProject],
  );

  const handleProjectVariablesBatchUpdate = useCallback(
    async (input: {
      projectId: string;
      creates?: {
        name: string;
        defaultValue: any;
        type: ProjectVariableType["type"];
        required: boolean;
        publicValue: boolean;
        index?: number;
        config?: AnyProjectVariable["config"];
      }[];
      updates?: {
        paramId: string;
        name?: string;
        defaultValue?: any;
        type?: ProjectVariableType["type"];
        required?: boolean;
        publicValue?: boolean;
        config?: AnyProjectVariable["config"];
      }[];
      deletes?: string[];
    }) => {
      await updateMultipleProjectVariables(input);
    },
    [updateMultipleProjectVariables],
  );

  const handleProjectVariableDelete = useCallback(
    async (id: string) => {
      if (!currentProject) return;

      try {
        await deleteProjectVariable(id, currentProject.id);
      } catch (error) {
        console.error("Failed to delete project variable:", error);
      }
    },
    [deleteProjectVariable, currentProject],
  );

  const handleProjectVariablesBatchDelete = useCallback(
    async (ids: string[]) => {
      if (!currentProject) return;

      try {
        await deleteProjectVariables(currentProject.id, ids);
      } catch (error) {
        console.error("Failed to delete project variables:", error);
      }
    },
    [deleteProjectVariables, currentProject],
  );

  const handleShowProjectVarsDialog = useCallback(() => {
    setShowProjectVarsDialog(true);
  }, []);

  const handleCloseProjectVarsDialog = useCallback(() => {
    setShowProjectVarsDialog(false);
  }, []);

  return (
    <div className="flex w-[100vw] shrink-0 justify-between gap-2 bg-secondary">
      <div className="flex items-center gap-1">
        <HomeMenu
          dropdownPosition="bottom"
          dropdownAlign="end"
          dropdownAlignOffset={-140}
        />
        <div className="pr-4 pl-2">
          <Breadcrumb />
        </div>
        <div className="flex items-center gap-2 rounded-md p-1">
          {/* <div className="border-r border-primary h-4/5" /> */}
          <IconButton
            className="h-[30px]"
            variant="outline"
            tooltipText={t("Project Variables")}
            icon={<ChalkboardTeacherIcon weight="thin" size={18} />}
            onClick={handleShowProjectVarsDialog}
          />
          <IconButton
            className="h-[30px]"
            variant="outline"
            tooltipText={t("Resources")}
            icon={<HardDriveIcon weight="thin" size={18} />}
            disabled
          />
          {/* <div className="border-r border-primary h-4/5" /> */}
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
      <div className="flex h-full items-center justify-center gap-2 self-center p-1 select-none">
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
          onProjectShare={onProjectShare}
          onProjectExport={onProjectExport}
          onWorkflowDeployment={onWorkflowDeployment}
        />
      </div>
      <ProjectVariableDialog
        isOpen={showProjectVarsDialog}
        currentProjectVariables={currentProjectVariables}
        onClose={handleCloseProjectVarsDialog}
        onAdd={handleProjectVariableAdd}
        onChange={handleProjectVariableChange}
        onDelete={handleProjectVariableDelete}
        onDeleteBatch={handleProjectVariablesBatchDelete}
        onBatchUpdate={handleProjectVariablesBatchUpdate}
        projectId={currentProject?.id}
      />
    </div>
  );
};

export default memo(TopBar);
