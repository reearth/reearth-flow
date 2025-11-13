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
        title: t("Workflow Variable Created"),
        description: t(
          "Workflow variable {{name}} has been created successfully.",
          { name },
        ),
      });

      return { projectVariable, ...rest };
    } catch (_err) {
      toast({
        title: t("Workflow Variable Creation Failed"),
        description: t("There was an error creating a workflow variable."),
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
          title: t("Workflow Variable Deleted"),
          description: t("Workflow variable has been deleted successfully."),
        });
        return { success: true, ...rest };
      } else {
        throw new Error("Delete operation returned false");
      }
    } catch (err) {
      console.error("Error deleting workflow variable:", err);
      toast({
        title: t("Workflow Variable Deletion Failed"),
        description: t("There was an error deleting a workflow variable."),
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
        title: t("Workflow Variables Updated"),
        description: t("Workflow variables have been updated successfully."),
      });

      return { projectVariables, ...rest };
    } catch (err) {
      console.error("Error updating workflow variables:", err);
      toast({
        title: t("Workflow Variables Update Failed"),
        description: t("There was an error updating workflow variables."),
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
          title: t("Workflow Variables Deleted"),
          description: t("Workflow variables have been deleted successfully."),
        });
        return { success: true, ...rest };
      } else {
        throw new Error("Batch delete operation returned false");
      }
    } catch (err) {
      console.error("Error deleting workflow variables:", err);
      toast({
        title: t("Workflow Variable Deletion Failed"),
        description: t("There was an error deleting workflow variables."),
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
