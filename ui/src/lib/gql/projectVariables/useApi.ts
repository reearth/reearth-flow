import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type { CreateProjectVariable, VarType } from "@flow/types";

import { useQueries } from "./useQueries";

export const useProjectVariables = () => {
  const {
    useProjectVariablesQuery,
    createProjectVariablesMutation,
    updateMultipleProjectVariablesMutation,
    deleteProjectVariableMutation,
    deleteProjectVariablesMutation,
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
    config?: any,
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
        config,
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

  const deleteProjectVariable = async (paramId: string, projectId: string) => {
    const { mutateAsync, ...rest } = deleteProjectVariableMutation;
    try {
      const result = await mutateAsync({ paramId, projectId });
      if (result?.success) {
        toast({
          title: t("Project Variable Deleted"),
          description: t("Project variable has been deleted successfully."),
        });
        return { success: true, ...rest };
      } else {
        throw new Error("Delete operation returned false");
      }
    } catch (err) {
      console.error("Error deleting project variable:", err);
      toast({
        title: t("Project Variable Deletion Failed"),
        description: t("There was an error deleting a project variable."),
        variant: "warning",
      });
      return { success: false, ...rest };
    }
  };

  const updateMultipleProjectVariables = async (input: {
    projectId: string;
    creates?: {
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
      index?: number;
      config?: any;
    }[];
    updates?: {
      paramId: string;
      name?: string;
      defaultValue?: any;
      type?: VarType;
      required?: boolean;
      publicValue?: boolean;
      config?: any;
    }[];
    deletes?: string[];
    reorders?: {
      paramId: string;
      newIndex: number;
    }[];
  }) => {
    const { mutateAsync, ...rest } = updateMultipleProjectVariablesMutation;
    try {
      const projectVariables = await mutateAsync(input);

      toast({
        title: t("Project Variables Updated"),
        description: t("Project variables have been updated successfully."),
      });

      return { projectVariables, ...rest };
    } catch (err) {
      console.error("Error updating project variables:", err);
      toast({
        title: t("Project Variables Update Failed"),
        description: t("There was an error updating project variables."),
        variant: "warning",
      });
      return { projectVariables: [], ...rest };
    }
  };

  const deleteProjectVariables = async (
    projectId: string,
    paramIds: string[],
  ) => {
    const { mutateAsync, ...rest } = deleteProjectVariablesMutation;
    try {
      const result = await mutateAsync({ paramIds, projectId });
      if (result?.success) {
        toast({
          title: t("Project Variables Deleted"),
          description: t("Project variables have been deleted successfully."),
        });
        return { success: true, ...rest };
      } else {
        throw new Error("Batch delete operation returned false");
      }
    } catch (err) {
      console.error("Error deleting project variables:", err);
      toast({
        title: t("Project Variable Deletion Failed"),
        description: t("There was an error deleting project variables."),
        variant: "warning",
      });
      return { success: false, ...rest };
    }
  };

  return {
    useGetProjectVariables,
    createProjectVariable,
    updateMultipleProjectVariables,
    deleteProjectVariable,
    deleteProjectVariables,
  };
};
