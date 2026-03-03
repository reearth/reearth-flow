import { useCallback, useMemo, useState } from "react";

import { useWorkflowVariables } from "@flow/lib/gql";
import { useCurrentProject } from "@flow/stores";
import {
  WorkflowVariable as WorkflowVariableType,
  AnyWorkflowVariable,
} from "@flow/types";

import { DialogOptions } from "../../types";

export default () => {
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);

  const {
    useGetWorkflowVariables,
    createWorkflowVariable,
    updateMultipleWorkflowVariables,
    deleteWorkflowVariable,
    deleteWorkflowVariables,
  } = useWorkflowVariables();
  const [currentProject] = useCurrentProject();
  const { workflowVariables, refetch: refetchWorkflowVariables } =
    useGetWorkflowVariables(currentProject?.id);

  const currentWorkflowVariables = useMemo(
    () => workflowVariables ?? [],
    [workflowVariables],
  );

  const handleDialogOpen = (dialog: DialogOptions) => {
    if (dialog === "workflowVariables") {
      refetchWorkflowVariables();
    }
    setShowDialog(dialog);
  };

  const handleDialogClose = () => setShowDialog(undefined);

  const handleWorkflowVariableAdd = useCallback(
    async (workflowVariable: WorkflowVariableType) => {
      if (!currentProject) return;

      await createWorkflowVariable(
        currentProject.id,
        workflowVariable.name,
        workflowVariable.defaultValue,
        workflowVariable.type,
        workflowVariable.required,
        workflowVariable.public,
        currentWorkflowVariables.length,
        workflowVariable.config,
      );
    },
    [currentProject, createWorkflowVariable, currentWorkflowVariables.length],
  );

  const handleWorkflowVariableChange = useCallback(
    async (workflowVariable: WorkflowVariableType) => {
      if (!currentProject) return;

      await updateMultipleWorkflowVariables({
        projectId: currentProject.id,
        updates: [
          {
            paramId: workflowVariable.id,
            name: workflowVariable.name,
            defaultValue: workflowVariable.defaultValue,
            type: workflowVariable.type,
            required: workflowVariable.required,
            publicValue: workflowVariable.public,
            config: workflowVariable.config,
          },
        ],
      });
    },
    [updateMultipleWorkflowVariables, currentProject],
  );

  const handleWorkflowVariablesBatchUpdate = useCallback(
    async (input: {
      projectId: string;
      creates?: {
        name: string;
        defaultValue: any;
        type: WorkflowVariableType["type"];
        required: boolean;
        publicValue: boolean;
        index?: number;
        config?: AnyWorkflowVariable["config"];
      }[];
      updates?: {
        paramId: string;
        name?: string;
        defaultValue?: any;
        type?: WorkflowVariableType["type"];
        required?: boolean;
        publicValue?: boolean;
        config?: AnyWorkflowVariable["config"];
      }[];
      deletes?: string[];
    }) => {
      await updateMultipleWorkflowVariables(input);
    },
    [updateMultipleWorkflowVariables],
  );

  const handleWorkflowVariableDelete = useCallback(
    async (id: string) => {
      if (!currentProject) return;

      try {
        await deleteWorkflowVariable(id, currentProject.id);
      } catch (error) {
        console.error("Failed to delete workflow variable:", error);
      }
    },
    [deleteWorkflowVariable, currentProject],
  );

  const handleWorkflowVariablesBatchDelete = useCallback(
    async (ids: string[]) => {
      if (!currentProject) return;

      try {
        await deleteWorkflowVariables(currentProject.id, ids);
      } catch (error) {
        console.error("Failed to delete workflow variables:", error);
      }
    },
    [deleteWorkflowVariables, currentProject],
  );

  return {
    showDialog,
    currentProject,
    currentWorkflowVariables,
    handleWorkflowVariableAdd,
    handleWorkflowVariableChange,
    handleWorkflowVariablesBatchUpdate,
    handleWorkflowVariableDelete,
    handleWorkflowVariablesBatchDelete,
    handleDialogOpen,
    handleDialogClose,
  };
};
