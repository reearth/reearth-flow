import { useCallback, useEffect, useState } from "react";
import { UndoManager } from "yjs";

// const historyClientPrepend = "undo-redo-operation";

export default ({
  undoManager,
  globalWorkflowsUndoManager,
  // undoTrackerActionWrapper,
}: {
  undoManager: UndoManager | null;
  globalWorkflowsUndoManager: UndoManager | null;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
}) => {
  const [canUndo, setCanUndo] = useState<boolean>(false);
  const [canRedo, setCanRedo] = useState<boolean>(false);

  useEffect(() => {
    const updateCanUndo = () => {
      const workflowStackLength = undoManager?.undoStack?.length ?? 0;
      const globalStackLength =
        globalWorkflowsUndoManager?.undoStack?.length ?? 0;
      setCanUndo(workflowStackLength > 0 || globalStackLength > 0);
    };

    const updateCanRedo = () => {
      const workflowStackLength = undoManager?.redoStack?.length ?? 0;
      const globalStackLength =
        globalWorkflowsUndoManager?.redoStack?.length ?? 0;
      setCanRedo(workflowStackLength > 0 || globalStackLength > 0);
    };

    const updateStacks = () => {
      updateCanUndo();
      updateCanRedo();
    };

    // Initial update
    updateStacks();

    // Listen to stack changes
    if (undoManager) {
      undoManager.on("stack-item-added", updateStacks);
      undoManager.on("stack-item-popped", updateStacks);
    }

    if (globalWorkflowsUndoManager) {
      globalWorkflowsUndoManager.on("stack-item-added", updateStacks);
      globalWorkflowsUndoManager.on("stack-item-popped", updateStacks);
    }

    return () => {
      if (undoManager) {
        undoManager.off("stack-item-added", updateStacks);
        undoManager.off("stack-item-popped", updateStacks);
      }

      if (globalWorkflowsUndoManager) {
        globalWorkflowsUndoManager.off("stack-item-added", updateStacks);
        globalWorkflowsUndoManager.off("stack-item-popped", updateStacks);
      }
    };
  }, [undoManager, globalWorkflowsUndoManager]);

  const handleYWorkflowUndo = useCallback(() => {
    const workflowStackLength = undoManager?.undoStack?.length ?? 0;
    const globalStackLength =
      globalWorkflowsUndoManager?.undoStack?.length ?? 0;

    if (workflowStackLength > 0 || globalStackLength > 0) {
      try {
        // Undo both workflow-specific changes and global workflow map changes
        if (workflowStackLength > 0) {
          undoManager?.undo();
        }
        if (globalStackLength > 0) {
          globalWorkflowsUndoManager?.undo();
        }
      } catch (e) {
        console.error("Undo operation failed: ", e);

        if (workflowStackLength > 0) {
          undoManager?.undoStack.splice(undoManager?.undoStack.length - 1, 1);
        }
        if (globalStackLength > 0) {
          globalWorkflowsUndoManager?.undoStack.splice(
            globalWorkflowsUndoManager?.undoStack.length - 1,
            1,
          );
        }

        if (
          (undoManager?.undoStack.length ?? 0) > 0 ||
          (globalWorkflowsUndoManager?.undoStack.length ?? 0) > 0
        ) {
          setTimeout(handleYWorkflowUndo, 0);
        }
      }
    }
  }, [undoManager, globalWorkflowsUndoManager]);

  const handleYWorkflowRedo = useCallback(() => {
    const workflowStackLength = undoManager?.redoStack?.length ?? 0;
    const globalStackLength =
      globalWorkflowsUndoManager?.redoStack?.length ?? 0;

    if (workflowStackLength > 0 || globalStackLength > 0) {
      try {
        // Redo both workflow-specific changes and global workflow map changes
        if (globalStackLength > 0) {
          globalWorkflowsUndoManager?.redo();
        }
        if (workflowStackLength > 0) {
          undoManager?.redo();
        }
      } catch (e) {
        console.error("Redo operation failed: ", e);

        if (workflowStackLength > 0) {
          undoManager?.redoStack.splice(undoManager?.redoStack.length - 1, 1);
        }
        if (globalStackLength > 0) {
          globalWorkflowsUndoManager?.redoStack.splice(
            globalWorkflowsUndoManager?.redoStack.length - 1,
            1,
          );
        }

        if (
          (undoManager?.redoStack.length ?? 0) > 0 ||
          (globalWorkflowsUndoManager?.redoStack.length ?? 0) > 0
        ) {
          setTimeout(handleYWorkflowRedo, 0);
        }
      }
    }
  }, [undoManager, globalWorkflowsUndoManager]);

  return {
    canUndo,
    canRedo,
    handleYWorkflowUndo,
    handleYWorkflowRedo,
  };
};
