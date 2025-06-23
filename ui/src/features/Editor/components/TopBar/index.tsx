import { ChalkboardTeacherIcon, HardDriveIcon } from "@phosphor-icons/react";
import { memo, useState, useCallback, useMemo, useEffect } from "react";
import { Doc } from "yjs";

import { IconButton } from "@flow/components";
import { useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import {
  ProjectVariable as ProjectVariableType,
  VarType,
  Project,
} from "@flow/types";
import { getDefaultValueForProjectVar } from "@flow/utils";

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
    updateProjectVariable,
    deleteProjectVariable,
  } = useProjectVariables();

  const { projectVariables } = useGetProjectVariables(currentProject?.id);

  const initialProjectVariables = useMemo(
    () => projectVariables ?? [],
    [projectVariables],
  );

  const [updatedProjectVariables, setUpdatedProjectVariables] = useState<
    ProjectVariableType[]
  >(initialProjectVariables);

  useEffect(() => {
    setUpdatedProjectVariables(initialProjectVariables);
  }, [initialProjectVariables]);

  const handleProjectVariableAdd = useCallback(
    async (type: VarType) => {
      if (!currentProject) return;
      const defaultValue = getDefaultValueForProjectVar(type);

      const res = await createProjectVariable(
        currentProject.id,
        t("New Project Variable"),
        defaultValue,
        type,
        true,
        true,
        updatedProjectVariables.length,
      );

      if (!res.projectVariable) return;

      const newProjectVariable = res.projectVariable;

      setUpdatedProjectVariables((prev) => [...prev, newProjectVariable]);
    },
    [currentProject, createProjectVariable, t, updatedProjectVariables.length],
  );

  const handleProjectVariableChange = useCallback(
    async (projectVariable: ProjectVariableType) => {
      await updateProjectVariable(
        projectVariable.id,
        projectVariable.name,
        projectVariable.defaultValue,
        projectVariable.type,
        projectVariable.required,
        projectVariable.public,
      );

      setUpdatedProjectVariables((prev) =>
        prev.map((variable) =>
          variable.id === projectVariable.id ? projectVariable : variable,
        ),
      );
    },
    [updateProjectVariable],
  );

  const handleProjectVariableDelete = useCallback(
    async (id: string) => {
      try {
        const result = await deleteProjectVariable(id);

        if (result.success) {
          setUpdatedProjectVariables((prev) =>
            prev.filter((variable) => variable.id !== id),
          );
        }
      } catch (error) {
        console.error("Failed to delete project variable:", error);
      }
    },
    [deleteProjectVariable],
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
        currentProjectVariables={updatedProjectVariables}
        onClose={handleCloseProjectVarsDialog}
        onAdd={handleProjectVariableAdd}
        onChange={handleProjectVariableChange}
        onDelete={handleProjectVariableDelete}
      />
    </div>
  );
};

export default memo(TopBar);
