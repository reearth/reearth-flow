import { useState } from "react";

import { Button, ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { projectVariables as MOCK_DATA } from "@flow/mock_data/projectVars";
import { ProjectVar } from "@flow/types";

import { ProjectVarDialog } from "./ProjectVarDialog";
import { ProjectVariable } from "./ProjectVariable";

const ProjectVariables: React.FC = () => {
  const t = useT();
  const [isOpen, setIsOpen] = useState(false);

  // TODO: get project variables from gql
  const [currentProjectVars, setCurrentProjectVars] =
    useState<ProjectVar[]>(MOCK_DATA);

  const handleDialogOpen = () => setIsOpen(true);
  const handleDialogClose = () => setIsOpen(false);

  const handleSubmit = (newProjectVars: ProjectVar[]) => {
    setCurrentProjectVars(newProjectVars);
    handleDialogClose();
  };

  return (
    <>
      <div className="flex h-full flex-col gap-4 p-1">
        <div className="flex-1">
          <div className="flex items-center pb-2">
            <p className="flex-1 text-sm font-thin">{t("Key")}</p>
            <p className="flex-1 text-sm font-thin">{t("Value")}</p>
          </div>
          <ScrollArea>
            <div className="flex flex-col gap-1 overflow-y-auto">
              {currentProjectVars.map((variable, idx) => (
                <ProjectVariable
                  key={variable.name + idx}
                  className={`${idx % 2 !== 0 ? "bg-card" : "bg-primary"}`}
                  variable={variable}
                />
              ))}
            </div>
          </ScrollArea>
        </div>
        <div className="flex justify-end">
          <Button
            className="self-end"
            size="sm"
            variant="outline"
            onClick={handleDialogOpen}>
            {t("Edit Project Variables")}
          </Button>
        </div>
      </div>
      <ProjectVarDialog
        isOpen={isOpen}
        currentProjectVars={currentProjectVars}
        onClose={handleDialogClose}
        onSubmit={handleSubmit}
      />
    </>
  );
};

export { ProjectVariables };
