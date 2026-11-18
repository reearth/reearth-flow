import { useState, useEffect, useCallback, useRef } from "react";

import type { WorkflowVariable } from "@flow/types";

export default ({
  variable,
  onClose,
  onUpdate,
  onLiveUpdate,
}: {
  variable: WorkflowVariable | null;
  onClose: () => void;
  onUpdate: (variable: WorkflowVariable) => void;
  onLiveUpdate?: (variable: WorkflowVariable) => void;
}) => {
  const [showAssetPicker, setShowAssetPicker] = useState(false);
  const [assetUrl, setAssetUrl] = useState<string | null>(null);
  const handleDialogOpen = () => setShowAssetPicker(true);
  const handleDialogClose = () => setShowAssetPicker(false);
  const [localVariable, setLocalVariable] = useState<WorkflowVariable | null>(
    null,
  );
  const [hasChanges, setHasChanges] = useState(false);

  // Snapshot of the variable when this edit session opened — used to revert on cancel.
  const openedVariableRef = useRef<WorkflowVariable | null>(null);

  const handleAssetSelect = (url: string) => {
    if (localVariable && variable) {
      const updated = { ...localVariable, defaultValue: url };
      setLocalVariable(updated);
      setHasChanges(true);
      onLiveUpdate?.(updated);
    }
    setAssetUrl(url);
  };

  // Sync from parent when no local edits in progress (passive viewer update).
  // Also captures the opening snapshot the first time variable becomes non-null.
  useEffect(() => {
    if (variable) {
      if (
        openedVariableRef.current &&
        openedVariableRef.current.id !== variable.id
      ) {
        // Variable ID changed while dialog was open — reset edit state entirely.
        openedVariableRef.current = { ...variable };
        setHasChanges(false);
        setLocalVariable({ ...variable });
      } else if (!hasChanges) {
        setLocalVariable({ ...variable });
        if (!openedVariableRef.current) {
          openedVariableRef.current = { ...variable };
        }
      }
    } else {
      setLocalVariable(null);
      setHasChanges(false);
      openedVariableRef.current = null;
    }
  }, [variable, hasChanges]);

  const handleFieldUpdate = useCallback(
    (updatedVariable: WorkflowVariable) => {
      setLocalVariable(updatedVariable);
      setHasChanges(true);
      onLiveUpdate?.(updatedVariable);
    },
    [onLiveUpdate],
  );

  const handleSave = useCallback(() => {
    if (localVariable && hasChanges) {
      onUpdate(localVariable);
    }
    openedVariableRef.current = null;
    onClose();
  }, [localVariable, hasChanges, onUpdate, onClose]);

  const handleCancel = useCallback(() => {
    // Revert any live Yjs writes back to the state when this edit session opened.
    if (hasChanges && openedVariableRef.current) {
      onLiveUpdate?.(openedVariableRef.current);
    }
    openedVariableRef.current = null;
    setHasChanges(false);
    onClose();
  }, [hasChanges, onLiveUpdate, onClose]);

  const clearUrl = () => {
    setAssetUrl(null);
  };

  return {
    localVariable,
    hasChanges,
    showAssetPicker,
    assetUrl,
    handleAssetSelect,
    handleFieldUpdate,
    handleSave,
    handleCancel,
    handleDialogOpen,
    handleDialogClose,
    clearUrl,
  };
};
