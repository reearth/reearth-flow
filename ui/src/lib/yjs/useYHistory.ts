import { useCallback, useMemo } from "react";
import { UndoManager } from "yjs";

export default ({ undoManager }: { undoManager: UndoManager | null }) => {
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
      undoManager?.undo();
    }
  }, [undoManager]);

  const handleYWorkflowRedo = useCallback(() => {
    const stackLength = undoManager?.redoStack?.length ?? 0;
    if (stackLength > 0) {
      undoManager?.redo();
    }
  }, [undoManager]);

  return {
    canUndo,
    canRedo,
    handleYWorkflowUndo,
    handleYWorkflowRedo,
  };
};
