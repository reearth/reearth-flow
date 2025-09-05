import { useCallback, useMemo } from "react";
import { UndoManager } from "yjs";

// const historyClientPrepend = "undo-redo-operation";

export default ({
  undoManager,
  // undoTrackerActionWrapper,
}: {
  undoManager: UndoManager | null;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
}) => {
  const canUndo = useMemo(() => {
    const stackLength = undoManager?.undoStack?.length ?? 0;
    return stackLength > 0;
  }, [undoManager?.undoStack?.length]);

  const canRedo = useMemo(() => {
    const stackLength = undoManager?.redoStack?.length ?? 0;
    return stackLength > 0;
  }, [undoManager?.redoStack?.length]);

  const handleYWorkflowUndo = useCallback(() => {
    const stackLength = undoManager?.undoStack?.length ?? 0;
    if (stackLength > 0) {
      try {
        // undoTrackerActionWrapper(() => {
        undoManager?.undo();
        // }, historyClientPrepend);
      } catch (e) {
        console.error("Undo operation failed: ", e);

        if (undoManager && undoManager.undoStack) {
          undoManager.undoStack.splice(undoManager.undoStack.length - 1, 1);
        }

        if (undoManager?.undoStack.length) {
          setTimeout(handleYWorkflowUndo, 0);
        }
      }
    }
  }, [undoManager]);

  const handleYWorkflowRedo = useCallback(() => {
    const stackLength = undoManager?.redoStack?.length ?? 0;
    if (stackLength > 0) {
      try {
        // undoTrackerActionWrapper(() => {
        undoManager?.redo();
        // }, historyClientPrepend);
      } catch (e) {
        console.error("Redo operation failed: ", e);

        if (undoManager && undoManager.redoStack) {
          undoManager.redoStack.splice(undoManager.redoStack.length - 1, 1);
        }

        if (undoManager?.redoStack.length) {
          setTimeout(handleYWorkflowRedo, 0);
        }
      }
    }
  }, [undoManager]);

  return {
    canUndo,
    canRedo,
    handleYWorkflowUndo,
    handleYWorkflowRedo,
  };
};
