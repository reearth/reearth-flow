import { useState } from "react";

import { Button, ScrollArea } from "@flow/components";
import { useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import { ProjectVariable as ProjectVariableType } from "@flow/types";

import { ProjectVariable } from "./ProjectVariable";
import { ProjectVariableDialog } from "./ProjectVariableDialog";

const ProjectVariables: React.FC = () => {
  const t = useT();
  const [isOpen, setIsOpen] = useState(false);

  const [currentProject] = useCurrentProject();

  const { useGetProjectVariables, createProjectVariable } =
    useProjectVariables();

  const { projectVariables } = useGetProjectVariables(currentProject?.id);

  const [updatedProjectVariables, setUpdatedProjectVariables] = useState<
    ProjectVariableType[]
  >(projectVariables ?? []);

  const handleDialogOpen = () => setIsOpen(true);
  const handleDialogClose = () => setIsOpen(false);

  const handleSubmit = async (newProjectVariables: ProjectVariableType[]) => {
    if (!currentProject) return;

    await (async () => {
      try {
        newProjectVariables.forEach(async (projectVar) => {
          const { name, value, type, required } = projectVar;
          const index = updatedProjectVariables.length;
          const existingVariable = updatedProjectVariables.find(
            (p) => p.name === name,
          );
          if (existingVariable) {
            // Update existing parameter
          } else {
            await createProjectVariable(
              currentProject.id,
              name,
              value,
              type,
              required,
              index,
            );
          }
        });
      } catch (error) {
        console.error("Error creating project variable", error);
      }
    })();

    setUpdatedProjectVariables(newProjectVariables);
    handleDialogClose();
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
          currentProjectVariable={projectVariables}
          onClose={handleDialogClose}
          onSubmit={handleSubmit}
        />
      )}
    </>
  );
};

export { ProjectVariables };
