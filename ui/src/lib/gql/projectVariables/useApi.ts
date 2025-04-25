import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateProjectVariable,
  UpdateProjectVariable,
  VarType,
} from "@flow/types";

import { useQueries } from "./useQueries";

export const useProjectVariables = () => {
  const {
    useProjectVariablesQuery,
    createProjectVariablesMutation,
    updateProjectVariablesMutation,
    deleteProjectVariableMutation,
  } = useQueries();

  const { toast } = useToast();
  const t = useT();

  const useGetProjectVariables = (projectId?: string) => {
    const { data, ...rest } = useProjectVariablesQuery(projectId);
    return {
      projectVariables: data,
      ...rest,
    };
  };

  const createProjectVariable = async (
    projectId: string,
    name: string,
    defaultValue: any,
    type: VarType,
    required: boolean,
    publicValue: boolean,
    index: number,
  ): Promise<CreateProjectVariable> => {
    const { mutateAsync, ...rest } = createProjectVariablesMutation;
    try {
      const projectVariable = await mutateAsync({
        projectId,
        name,
        defaultValue,
        type,
        required,
        publicValue,
        index,
      });

      toast({
        title: t("Project Variable Created"),
        description: t(
          "Project variable {{name}} has been created successfully.",
          { name },
        ),
      });

      return { projectVariable, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Variable Creation Failed"),
        description: t("There was an error creating a project variable."),
        variant: "warning",
      });

      return { projectVariable: undefined, ...rest };
    }
  };

  const updateProjectVariable = async (
    paramId: string,
    name: string,
    defaultValue: any,
    type: VarType,
    required: boolean,
    publicValue: boolean,
  ): Promise<UpdateProjectVariable> => {
    const { mutateAsync, ...rest } = updateProjectVariablesMutation;

    try {
      const projectVariable = await mutateAsync({
        paramId,
        name,
        defaultValue,
        type,
        required,
        publicValue,
      });

      toast({
        title: t("Project Variable Updated"),
        description: t("Project variable has been updated successfully."),
      });

      return { projectVariable, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Variable Update Failed"),
        description: t("There was an error updating a project variable."),
        variant: "warning",
      });

      return { projectVariable: undefined, ...rest };
    }
  };

  const deleteProjectVariable = async (paramId: string) => {
    const { mutateAsync, ...rest } = deleteProjectVariableMutation;
    try {
      await mutateAsync(paramId);
      toast({
        title: t("Project Variable Deleted"),
        description: t("Project variable has been deleted successfully."),
      });
      return { success: true, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Variable Deletion Failed"),
        description: t("There was an error deleting a project variable."),
        variant: "warning",
      });
      return { success: false, ...rest };
    }
  };

  return {
    useGetProjectVariables,
    createProjectVariable,
    updateProjectVariable,
    deleteProjectVariable,
  };
};
