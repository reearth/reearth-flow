import { useState, useEffect, useCallback } from "react";

import { useWorkflowVars } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import { WorkflowVariable, VarType } from "@flow/types";
import {
  generateUUID,
  getDefaultConfigForWorkflowVar,
  getDefaultValueForWorkflowVar,
} from "@flow/utils";

type PendingChange =
  | {
      type: "add";
      tempId: string;
      workflowVariable: WorkflowVariable;
    }
  | {
      type: "update";
      workflowVariable: WorkflowVariable;
    }
  | {
      type: "delete";
      id: string;
    }
  | {
      type: "reorder";
      paramId: string;
      newIndex: number;
    };

export default ({
  currentWorkflowVariables,
  projectId,
  onClose,
  onAdd,
  onChange,
  onDelete,
  onDeleteBatch,
  onBatchUpdate,
}: {
  currentWorkflowVariables?: WorkflowVariable[];
  projectId?: string;
  onClose: () => void;
  onAdd: (workflowVariable: WorkflowVariable) => Promise<void>;
  onChange: (workflowVariable: WorkflowVariable) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onDeleteBatch?: (ids: string[]) => Promise<void>;
  onBatchUpdate?: (input: {
    projectId: string;
    creates?: {
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
      index?: number;
    }[];
    updates?: {
      paramId: string;
      name?: string;
      defaultValue?: any;
      type?: VarType;
      required?: boolean;
      publicValue?: boolean;
    }[];
    deletes?: string[];
    reorders?: {
      paramId: string;
      newIndex: number;
    }[];
  }) => Promise<void>;
}) => {
  const t = useT();
  const [localWorkflowVariables, setLocalWorkflowVariables] = useState<
    WorkflowVariable[]
  >([]);
  const [pendingChanges, setPendingChanges] = useState<PendingChange[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [editingVariable, setEditingVariable] =
    useState<WorkflowVariable | null>(null);

  const { getUserFacingName } = useWorkflowVars();

  useEffect(() => {
    if (currentWorkflowVariables) {
      setLocalWorkflowVariables([...currentWorkflowVariables]);
      setPendingChanges([]);
    }
  }, [currentWorkflowVariables, getUserFacingName]);

  const handleLocalAdd = useCallback(
    (type: VarType) => {
      const tempId = `temp_${generateUUID()}`;
      const newVariable: WorkflowVariable = {
        id: tempId,
        name: t("New Workflow Variable"),
        defaultValue: getDefaultValueForWorkflowVar(type),
        config: getDefaultConfigForWorkflowVar(type),
        type,
        required: true,
        public: true,
      };

      setLocalWorkflowVariables((prev) => [...prev, newVariable]);
      setPendingChanges((prev) => [
        ...prev,
        { type: "add", tempId, workflowVariable: newVariable },
      ]);
    },
    [t],
  );

  const handleLocalUpdate = useCallback((updatedVariable: WorkflowVariable) => {
    setLocalWorkflowVariables((prev) =>
      prev.map((variable) =>
        variable.id === updatedVariable.id ? updatedVariable : variable,
      ),
    );

    // Handle pending changes differently for new vs existing variables
    setPendingChanges((prev) => {
      // If this is a new variable (temp ID), update the "add" change
      if (updatedVariable.id.startsWith("temp_")) {
        const existingAddIndex = prev.findIndex(
          (change) =>
            change.type === "add" && change.tempId === updatedVariable.id,
        );

        if (existingAddIndex >= 0) {
          const newChanges = [...prev];
          newChanges[existingAddIndex] = {
            type: "add",
            tempId: updatedVariable.id,
            workflowVariable: updatedVariable,
          };
          return newChanges;
        }
      } else {
        // For existing variables, handle as update
        const existingUpdateIndex = prev.findIndex(
          (change) =>
            change.type === "update" &&
            change.workflowVariable.id === updatedVariable.id,
        );

        if (existingUpdateIndex >= 0) {
          const newChanges = [...prev];
          newChanges[existingUpdateIndex] = {
            type: "update",
            workflowVariable: updatedVariable,
          };
          return newChanges;
        } else {
          return [
            ...prev,
            { type: "update", workflowVariable: updatedVariable },
          ];
        }
      }

      return prev;
    });
  }, []);

  const handleDeleteSingle = useCallback(
    (variableId: string) => {
      const variableIndex = localWorkflowVariables.findIndex(
        (variable) => variable.id === variableId,
      );

      if (variableIndex === -1) return;

      const varToDelete = localWorkflowVariables[variableIndex];

      setLocalWorkflowVariables((prev) =>
        prev.filter((variable) => variable.id !== variableId),
      );

      if (!varToDelete.id.startsWith("temp_")) {
        setPendingChanges((prev) => [
          ...prev,
          { type: "delete", id: varToDelete.id },
        ]);
      } else {
        setPendingChanges((prev) =>
          prev.filter(
            (change) =>
              !(change.type === "add" && change.tempId === varToDelete.id),
          ),
        );
      }
    },
    [localWorkflowVariables],
  );

  const handleReorder = useCallback(
    (oldIndex: number, newIndex: number) => {
      if (oldIndex === newIndex) return;

      const newWorkflowVariables = [...localWorkflowVariables];
      const [movedItem] = newWorkflowVariables.splice(oldIndex, 1);
      newWorkflowVariables.splice(newIndex, 0, movedItem);

      setLocalWorkflowVariables(newWorkflowVariables);

      // Track reorder changes for all affected non-temp variables
      newWorkflowVariables.forEach((variable, index) => {
        if (!variable.id.startsWith("temp_")) {
          const originalIndex = localWorkflowVariables.findIndex(
            (v) => v.id === variable.id,
          );
          if (originalIndex !== index) {
            setPendingChanges((prev) => {
              const filteredChanges = prev.filter(
                (change) =>
                  !(
                    change.type === "reorder" && change.paramId === variable.id
                  ),
              );
              return [
                ...filteredChanges,
                { type: "reorder", paramId: variable.id, newIndex: index },
              ];
            });
          }
        }
      });
    },
    [localWorkflowVariables],
  );

  const handleSubmit = useCallback(async () => {
    setIsSubmitting(true);
    try {
      const addChanges = pendingChanges.filter(
        (change) => change.type === "add",
      );
      const updateChanges = pendingChanges.filter(
        (change) => change.type === "update",
      );
      const deleteChanges = pendingChanges.filter(
        (change) => change.type === "delete",
      );
      const reorderChanges = pendingChanges.filter(
        (change) => change.type === "reorder",
      );

      if (
        onBatchUpdate &&
        projectId &&
        (addChanges.length > 0 ||
          updateChanges.length > 0 ||
          deleteChanges.length > 0 ||
          reorderChanges.length > 0)
      ) {
        const creates = addChanges.map((change) => ({
          name: change.workflowVariable.name,
          defaultValue: change.workflowVariable.defaultValue,
          config: change.workflowVariable.config,
          type: change.workflowVariable.type,
          required: change.workflowVariable.required,
          publicValue: change.workflowVariable.public,
          index: localWorkflowVariables.length,
        }));

        const updates = updateChanges.map((change) => ({
          paramId: change.workflowVariable.id,
          name: change.workflowVariable.name,
          defaultValue: change.workflowVariable.defaultValue,
          config: change.workflowVariable.config,
          type: change.workflowVariable.type,
          required: change.workflowVariable.required,
          publicValue: change.workflowVariable.public,
        }));

        const deletes = deleteChanges.map((change) => change.id);

        const reorders = reorderChanges.map((change) => ({
          paramId: change.paramId,
          newIndex: change.newIndex,
        }));

        await onBatchUpdate({
          projectId,
          ...(creates.length > 0 && { creates }),
          ...(updates.length > 0 && { updates }),
          ...(deletes.length > 0 && { deletes }),
          ...(reorders.length > 0 && { reorders }),
        });
      } else {
        for (const change of addChanges) {
          await onAdd(change.workflowVariable);
        }

        for (const change of updateChanges) {
          await onChange(change.workflowVariable);
        }

        if (deleteChanges.length > 0) {
          const deleteIds = deleteChanges.map((change) => change.id);

          if (onDeleteBatch && deleteChanges.length > 1) {
            await onDeleteBatch(deleteIds);
          } else {
            for (const change of deleteChanges) {
              await onDelete(change.id);
            }
          }
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 100));

      setPendingChanges([]);
      onClose();
    } catch (error) {
      console.error("Failed to submit workflow variable changes:", error);
    } finally {
      setIsSubmitting(false);
    }
  }, [
    pendingChanges,
    onBatchUpdate,
    projectId,
    localWorkflowVariables.length,
    onAdd,
    onChange,
    onDeleteBatch,
    onDelete,
    onClose,
  ]);

  const handleCancel = useCallback(() => {
    if (currentWorkflowVariables) {
      setLocalWorkflowVariables([...currentWorkflowVariables]);
    }
    setPendingChanges([]);
    onClose();
  }, [currentWorkflowVariables, onClose]);

  const handleEditVariable = useCallback((variable: WorkflowVariable) => {
    setEditingVariable(variable);
  }, []);

  const handleCloseEdit = useCallback(() => {
    setEditingVariable(null);
  }, []);

  return {
    localWorkflowVariables,
    pendingChanges,
    isSubmitting,
    editingVariable,
    getUserFacingName,
    handleLocalAdd,
    handleLocalUpdate,
    handleDeleteSingle,
    handleReorder,
    handleSubmit,
    handleCancel,
    handleEditVariable,
    handleCloseEdit,
  };
};
