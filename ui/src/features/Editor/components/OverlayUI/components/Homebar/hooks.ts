import { useCallback, useMemo, useState } from "react";

import { useWorkflowVariables } from "@flow/lib/gql";
import { useFollowSync } from "@flow/lib/yjs";
import { useCurrentProject } from "@flow/stores";
import {
  WorkflowVariable as WorkflowVariableType,
  AnyWorkflowVariable,
  type AwarenessDialog,
} from "@flow/types";

import { DialogOptions } from "../../types";

/** Dialogs whose open state lives in this hook rather than in the overlay. */
const OWNED_DIALOGS: readonly AwarenessDialog[] = [
  "workflowVariables",
  "assets",
  "collaboration",
];

/**
 * Subset that spotlight follows. The collaboration popover is excluded — it is
 * the control used to spotlight in the first place, so mirroring it is noise.
 */
const FOLLOWABLE_DIALOGS: readonly AwarenessDialog[] = [
  "workflowVariables",
  "assets",
];

export default ({
  onUserFocusedElement,
  onDialogAwareness,
  isFollowing = false,
  followDialog,
}: {
  onUserFocusedElement?: (isOpen: boolean) => void;
  onDialogAwareness?: (
    dialog: AwarenessDialog | null,
    ownedDialogs: readonly AwarenessDialog[],
  ) => void;
  isFollowing?: boolean;
  followDialog?: AwarenessDialog | null;
}) => {
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

  const handleDialogOpen = useCallback(
    (dialog: DialogOptions) => {
      if (dialog === "workflowVariables") {
        refetchWorkflowVariables();
      }
      setShowDialog(dialog);
      onUserFocusedElement?.(true);
      onDialogAwareness?.(dialog ?? null, OWNED_DIALOGS);
    },
    [refetchWorkflowVariables, onUserFocusedElement, onDialogAwareness],
  );

  const handleDialogClose = useCallback(() => {
    setShowDialog(undefined);
    onUserFocusedElement?.(false);
    onDialogAwareness?.(null, OWNED_DIALOGS);
  }, [onUserFocusedElement, onDialogAwareness]);

  const handleFollowDialog = useCallback(
    (dialog: AwarenessDialog | null) =>
      dialog ? handleDialogOpen(dialog) : handleDialogClose(),
    [handleDialogOpen, handleDialogClose],
  );

  const followable =
    followDialog && FOLLOWABLE_DIALOGS.includes(followDialog)
      ? followDialog
      : null;

  useFollowSync<AwarenessDialog>({
    active: isFollowing,
    follow: followable,
    local: showDialog && OWNED_DIALOGS.includes(showDialog) ? showDialog : null,
    onFollow: handleFollowDialog,
  });

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
        name: string;
        defaultValue: any;
        type: WorkflowVariableType["type"];
        required: boolean;
        publicValue: boolean;
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
