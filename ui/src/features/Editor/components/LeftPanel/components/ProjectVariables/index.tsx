import { useState } from "react";

import { Button, ScrollArea } from "@flow/components";
import { useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import { ProjectVariable as ProjectVariableType, VarType } from "@flow/types";
import { getDefaultValueForProjectVar } from "@flow/utils";

import { ProjectVariable } from "./ProjectVariable";
import { ProjectVariableDialog } from "./ProjectVariableDialog";

const ProjectVariables: React.FC = () => {
  const t = useT();
  const [isOpen, setIsOpen] = useState(false);

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

  const handleDialogOpen = () => setIsOpen(true);
  const handleDialogClose = () => setIsOpen(false);

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
    <>
      <div className="flex h-full flex-col gap-4 p-1">
        <div className="flex-1">
          <div className="flex items-center pb-2">
            <p className="flex-1 text-sm font-thin">{t("Name")}</p>
            <p className="flex-1 text-sm font-thin">{t("Value")}</p>
          </div>
          <ScrollArea>
            <div className="flex flex-col gap-1 overflow-y-auto">
              {projectVariables?.map((variable, idx) => (
                <ProjectVariable
                  key={variable.id}
                  className={`${idx % 2 !== 0 ? "bg-card" : "bg-primary"}`}
                  projectVariable={variable}
                />
              ))}
            </div>
          </ScrollArea>
        </div>
        <div className="flex justify-end">
          <Button
            className="self-end"
            size="sm"
            // variant="outline"
            onClick={handleDialogOpen}>
            {t("Edit")}
          </Button>
        </div>
      </div>
      {projectVariables && (
        <ProjectVariableDialog
          isOpen={isOpen}
          currentProjectVariables={projectVariables}
          onClose={handleDialogClose}
          onAdd={handleProjectVariableAdd}
          onChange={handleProjectVariableChange}
          onDelete={handleProjectVariableDelete}
        />
      )}
    </>
  );
};

export { ProjectVariables };
