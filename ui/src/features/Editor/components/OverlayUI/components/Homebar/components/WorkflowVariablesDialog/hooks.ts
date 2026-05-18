import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { useY } from "react-yjs";
import { Map as YMap } from "yjs";

import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useWorkflowVars } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import {
  WorkflowVarDraft,
  WorkflowVarDraftStore,
  getMostRecentOtherDraft,
} from "@flow/lib/yjs/workflowVarDrafts";
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
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
    }[];
    deletes?: string[];
    reorders?: {
      paramId: string;
      newIndex: number;
    }[];
  }) => Promise<void>;
}) => {
  const t = useT();
  const { yDoc, workflowVarAwareness } = useEditorContext();

  const myClientId = String(yDoc?.clientID ?? "local");

  const yVarDrafts = useMemo(
    () => yDoc?.getMap<WorkflowVarDraft>("workflowVarDrafts"),
    [yDoc],
  );
  const rawDrafts = useY(yVarDrafts ?? new YMap()) as WorkflowVarDraftStore;

  const [localWorkflowVariables, setLocalWorkflowVariables] = useState<
    WorkflowVariable[]
  >([]);
  const [pendingChanges, setPendingChanges] = useState<PendingChange[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [editingVariable, setEditingVariable] =
    useState<WorkflowVariable | null>(null);

  const { getUserFacingName } = useWorkflowVars();

  // Broadcast the current local variable list to Yjs so other users can see it
  const broadcastDraft = useCallback(
    (
      variables: WorkflowVariable[],
      currentEditingVariableId: string | null = null,
    ) => {
      if (!yVarDrafts) return;
      yVarDrafts.set(myClientId, {
        variables,
        timestamp: Date.now(),
        editingVariableId: currentEditingVariableId,
      });
    },
    [yVarDrafts, myClientId],
  );

  // Remove own draft from Yjs when dialog closes
  const removeDraft = useCallback(() => {
    if (!yVarDrafts) return;
    yVarDrafts.delete(myClientId);
  }, [yVarDrafts, myClientId]);

  // Initialize local state from server variables + any existing Yjs drafts
  useEffect(() => {
    if (currentWorkflowVariables) {
      setLocalWorkflowVariables([...currentWorkflowVariables]);
      setPendingChanges([]);
    }
  }, [currentWorkflowVariables]);

  // Track whether the dialog has been opened so we can broadcast on first render
  const hasOpenedRef = useRef(false);
  useEffect(() => {
    if (!hasOpenedRef.current && currentWorkflowVariables !== undefined) {
      hasOpenedRef.current = true;
      workflowVarAwareness?.onDialogOpen();
      broadcastDraft(currentWorkflowVariables, null);
    }
  }, [currentWorkflowVariables, workflowVarAwareness, broadcastDraft]);

  // Cleanup on unmount
  const removeDraftRef = useRef(removeDraft);
  removeDraftRef.current = removeDraft;
  const workflowVarAwarenessRef = useRef(workflowVarAwareness);
  workflowVarAwarenessRef.current = workflowVarAwareness;
  useEffect(() => {
    return () => {
      removeDraftRef.current();
      workflowVarAwarenessRef.current?.onDialogClose();
    };
  }, []);

  // Passive viewer sync: when another user changes something and we have no
  // pending edits of our own, update our display to reflect their live state
  useEffect(() => {
    if (pendingChanges.length > 0) return;
    const otherDraft = getMostRecentOtherDraft(myClientId, rawDrafts);
    if (otherDraft) {
      setLocalWorkflowVariables(otherDraft.variables);
    }
  }, [rawDrafts, myClientId, pendingChanges.length]);

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

      setLocalWorkflowVariables((prev) => {
        const next = [...prev, newVariable];
        broadcastDraft(next, editingVariable?.id ?? null);
        return next;
      });
      setPendingChanges((prev) => [
        ...prev,
        { type: "add", tempId, workflowVariable: newVariable },
      ]);
    },
    [t, broadcastDraft, editingVariable],
  );

  const handleLocalUpdate = useCallback(
    (updatedVariable: WorkflowVariable) => {
      setLocalWorkflowVariables((prev) => {
        const next = prev.map((variable) =>
          variable.id === updatedVariable.id ? updatedVariable : variable,
        );
        broadcastDraft(next, editingVariable?.id ?? null);
        return next;
      });

      setPendingChanges((prev) => {
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
    },
    [broadcastDraft, editingVariable],
  );

  const handleDeleteSingle = useCallback(
    (variableId: string) => {
      const variableIndex = localWorkflowVariables.findIndex(
        (variable) => variable.id === variableId,
      );

      if (variableIndex === -1) return;

      const varToDelete = localWorkflowVariables[variableIndex];

      setLocalWorkflowVariables((prev) => {
        const next = prev.filter((variable) => variable.id !== variableId);
        broadcastDraft(next, editingVariable?.id ?? null);
        return next;
      });

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
    [localWorkflowVariables, broadcastDraft, editingVariable],
  );

  const handleReorder = useCallback(
    (oldIndex: number, newIndex: number) => {
      if (oldIndex === newIndex) return;

      const newWorkflowVariables = [...localWorkflowVariables];
      const [movedItem] = newWorkflowVariables.splice(oldIndex, 1);
      newWorkflowVariables.splice(newIndex, 0, movedItem);

      broadcastDraft(newWorkflowVariables, editingVariable?.id ?? null);
      setLocalWorkflowVariables(newWorkflowVariables);

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
    [localWorkflowVariables, broadcastDraft, editingVariable],
  );

  // Called from VariableEditDialog on every field change (not just Save)
  const handleVariableLiveUpdate = useCallback(
    (updatedVariable: WorkflowVariable) => {
      setLocalWorkflowVariables((prev) => {
        const next = prev.map((v) =>
          v.id === updatedVariable.id ? updatedVariable : v,
        );
        broadcastDraft(next, updatedVariable.id);
        return next;
      });
      // Also keep editingVariable in sync so the dialog re-renders
      setEditingVariable(updatedVariable);
    },
    [broadcastDraft],
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
      removeDraft();
      workflowVarAwareness?.onDialogClose();
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
    removeDraft,
    workflowVarAwareness,
  ]);

  const handleCancel = useCallback(() => {
    if (currentWorkflowVariables) {
      setLocalWorkflowVariables([...currentWorkflowVariables]);
    }
    setPendingChanges([]);
    removeDraft();
    workflowVarAwareness?.onDialogClose();
    onClose();
  }, [currentWorkflowVariables, onClose, removeDraft, workflowVarAwareness]);

  const handleEditVariable = useCallback(
    (variable: WorkflowVariable) => {
      setEditingVariable(variable);
      workflowVarAwareness?.onEditStart(variable.id);
      broadcastDraft(localWorkflowVariables, variable.id);
    },
    [workflowVarAwareness, broadcastDraft, localWorkflowVariables],
  );

  const handleCloseEdit = useCallback(() => {
    setEditingVariable(null);
    workflowVarAwareness?.onEditStart(null);
    broadcastDraft(localWorkflowVariables, null);
  }, [workflowVarAwareness, broadcastDraft, localWorkflowVariables]);

  return {
    localWorkflowVariables,
    pendingChanges,
    isSubmitting,
    editingVariable,
    rawDrafts,
    getUserFacingName,
    handleLocalAdd,
    handleLocalUpdate,
    handleVariableLiveUpdate,
    handleDeleteSingle,
    handleReorder,
    handleSubmit,
    handleCancel,
    handleEditVariable,
    handleCloseEdit,
  };
};
