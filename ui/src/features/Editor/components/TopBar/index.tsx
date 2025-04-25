import { ChalkboardTeacher, HardDrive } from "@phosphor-icons/react";
import { memo, useState } from "react";

import { IconButton } from "@flow/components";
import { useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import { ProjectVariable as ProjectVariableType, VarType } from "@flow/types";
import { getDefaultValueForProjectVar } from "@flow/utils";

import { WorkflowTabs } from "..";
import { ProjectVariableDialog } from "../LeftPanel/components/ProjectVariables/ProjectVariableDialog";

import { ActionBar, Breadcrumb, DebugActionBar, HomeMenu } from "./components";

type Props = {
  currentWorkflowId: string;
  openWorkflows: {
    id: string;
    name: string;
  }[];
  allowedToDeploy: boolean;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  onProjectShare: (share: boolean) => void;
  onRightPanelOpen: (content?: "version-history") => void;
  onDebugRunStart: () => Promise<void>;
  onDebugRunStop: () => Promise<void>;
  onWorkflowClose: (workflowId: string) => void;
  onWorkflowChange: (workflowId?: string) => void;
  onWorkflowRename: (id: string, name: string) => void;
};

const TopBar: React.FC<Props> = ({
  currentWorkflowId,
  openWorkflows,
  allowedToDeploy,
  onWorkflowDeployment,
  onProjectShare,
  onRightPanelOpen,
  onDebugRunStart,
  onDebugRunStop,
  onWorkflowClose,
  onWorkflowChange,
  onWorkflowRename,
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

  const [updatedProjectVariables, setUpdatedProjectVariables] = useState<
    ProjectVariableType[]
  >(projectVariables ?? []);

  const handleProjectVariableAdd = async (type: VarType) => {
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
  };

  const handleProjectVariableChange = async (
    projectVariable: ProjectVariableType,
  ) => {
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
  };

  const handleProjectVariableDelete = async (id: string) => {
    await deleteProjectVariable(id);

    setUpdatedProjectVariables((prev) =>
      prev.filter((variable) => variable.id !== id),
    );
  };
  return (
    <div className="flex shrink-0 justify-between gap-2 bg-secondary w-[100vw]">
      <div className="flex items-center gap-1">
        <HomeMenu
          dropdownPosition="bottom"
          dropdownAlign="end"
          dropdownAlignOffset={-140}
        />
        <div className="pr-4 pl-2">
          <Breadcrumb />
        </div>
        <div className="flex gap-2 items-center p-1 rounded-md">
          {/* <div className="border-r border-primary h-4/5" /> */}
          <IconButton
            className="h-[30px]"
            variant="outline"
            tooltipText={t("Project Variables")}
            icon={<ChalkboardTeacher weight="thin" size={18} />}
            onClick={() => setShowProjectVarsDialog(true)}
          />
          <IconButton
            className="h-[30px]"
            variant="outline"
            tooltipText={t("Resources")}
            icon={<HardDrive weight="thin" size={18} />}
            disabled
          />
          {/* <div className="border-r border-primary h-4/5" /> */}
        </div>
      </div>
      <div className="flex flex-1 gap-2 h-full overflow-hidden">
        <WorkflowTabs
          currentWorkflowId={currentWorkflowId}
          openWorkflows={openWorkflows}
          onWorkflowClose={onWorkflowClose}
          onWorkflowChange={onWorkflowChange}
          onWorkflowRename={onWorkflowRename}
        />
      </div>
      <div className="flex select-none items-center h-full justify-center gap-2 self-center p-1">
        <div className="border-r h-4/5" />
        <DebugActionBar
          onDebugRunStart={onDebugRunStart}
          onDebugRunStop={onDebugRunStop}
        />
        <div className="border-r h-4/5" />
        <ActionBar
          allowedToDeploy={allowedToDeploy}
          onProjectShare={onProjectShare}
          onWorkflowDeployment={onWorkflowDeployment}
          onRightPanelOpen={onRightPanelOpen}
        />
      </div>
      <ProjectVariableDialog
        isOpen={showProjectVarsDialog}
        currentProjectVariables={projectVariables}
        onClose={() => setShowProjectVarsDialog(false)}
        onAdd={handleProjectVariableAdd}
        onChange={handleProjectVariableChange}
        onDelete={handleProjectVariableDelete}
      />
    </div>
  );
};

export default memo(TopBar);
