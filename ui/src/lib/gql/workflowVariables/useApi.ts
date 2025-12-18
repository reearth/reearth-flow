import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type { CreateWorkflowVariable, VarType } from "@flow/types";

import { useQueries } from "./useQueries";

export const useWorkflowVariables = () => {
  const {
    useWorkflowVariablesQuery,
    createWorkflowVariablesMutation,
    updateMultipleWorkflowVariablesMutation,
    deleteWorkflowVariableMutation,
    deleteWorkflowVariablesMutation,
  } = useQueries();

  const { toast } = useToast();
  const t = useT();

  const useGetWorkflowVariables = (projectId?: string) => {
    const { data, ...rest } = useWorkflowVariablesQuery(projectId);
    return {
      workflowVariables: data,
      ...rest,
    };
  };

  const createWorkflowVariable = async (
    projectId: string,
    name: string,
    defaultValue: any,
    type: VarType,
    required: boolean,
    publicValue: boolean,
    index: number,
    config?: any,
  ): Promise<CreateWorkflowVariable> => {
    const { mutateAsync, ...rest } = createWorkflowVariablesMutation;
    try {
      const workflowVariable = await mutateAsync({
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

      return { workflowVariable, ...rest };
    } catch (_err) {
      toast({
        title: t("Workflow Variable Creation Failed"),
        description: t("There was an error creating a workflow variable."),
        variant: "warning",
      });

      return { workflowVariable: undefined, ...rest };
    }
  };

  const deleteWorkflowVariable = async (paramId: string, projectId: string) => {
    const { mutateAsync, ...rest } = deleteWorkflowVariableMutation;
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

  const updateMultipleWorkflowVariables = async (input: {
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
    const { mutateAsync, ...rest } = updateMultipleWorkflowVariablesMutation;
    try {
      const workflowVariables = await mutateAsync(input);

      toast({
        title: t("Workflow Variables Updated"),
        description: t("Workflow variables have been updated successfully."),
      });

      return { workflowVariables, ...rest };
    } catch (err) {
      console.error("Error updating workflow variables:", err);
      toast({
        title: t("Workflow Variables Update Failed"),
        description: t("There was an error updating workflow variables."),
        variant: "warning",
      });
      return { workflowVariables: [], ...rest };
    }
  };

  const deleteWorkflowVariables = async (
    projectId: string,
    paramIds: string[],
  ) => {
    const { mutateAsync, ...rest } = deleteWorkflowVariablesMutation;
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
        title: t("Workflow Variables Deletion Failed"),
        description: t("There was an error deleting workflow variables."),
        variant: "warning",
      });
      return { success: false, ...rest };
    }
  };

  return {
    useGetWorkflowVariables,
    createWorkflowVariable,
    updateMultipleWorkflowVariables,
    deleteWorkflowVariable,
    deleteWorkflowVariables,
  };
};
