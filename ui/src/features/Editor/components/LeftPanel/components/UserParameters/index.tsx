import { useState } from "react";

import { Button, ScrollArea } from "@flow/components";
import { useUserParameter } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import { UserParameter as UserParameterType } from "@flow/types";

import { UserParameterDialog } from "./UserParamDialog";
import { UserParameter } from "./UserParameter";

const ProjectVariables: React.FC = () => {
  const t = useT();
  const [isOpen, setIsOpen] = useState(false);

  const [currentProject] = useCurrentProject();

  const { useGetUserParameters, createUserParameter } = useUserParameter();

  const { userParameters } = useGetUserParameters(currentProject?.id);

  const [updatedUserParameters, setCurrentUserParameters] = useState<
    UserParameterType[]
  >(userParameters ?? []);

  const handleDialogOpen = () => setIsOpen(true);
  const handleDialogClose = () => setIsOpen(false);

  const handleSubmit = async (newUserParameters: UserParameterType[]) => {
    if (!currentProject) return;

    await (async () => {
      try {
        newUserParameters.forEach(async (parameter) => {
          const { name, value, type, required } = parameter;
          const index = updatedUserParameters.length;
          const existingParameter = updatedUserParameters.find(
            (p) => p.name === name,
          );
          if (existingParameter) {
            // Update existing parameter
          } else {
            await createUserParameter(
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
        console.error("Error creating user parameters", error);
      }
    })();

    setCurrentUserParameters(newUserParameters);
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
              {userParameters?.map((parameter, idx) => (
                <UserParameter
                  key={parameter.id}
                  className={`${idx % 2 !== 0 ? "bg-card" : "bg-primary"}`}
                  parameter={parameter}
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
            {t("Edit User Parameters")}
          </Button>
        </div>
      </div>
      {userParameters && (
        <UserParameterDialog
          isOpen={isOpen}
          currentUserParameters={userParameters}
          onClose={handleDialogClose}
          onSubmit={handleSubmit}
        />
      )}
    </>
  );
};

export { ProjectVariables };
