import { useState, useEffect, useCallback, useRef } from "react";

import { Asset, WorkflowVariable } from "@flow/types";

export type DialogOptions = "assets" | "cms" | undefined;

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
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const [assetUrl, setAssetUrl] = useState<string | null>(null);
  const handleDialogOpen = (dialog: DialogOptions) => setShowDialog(dialog);
  const handleDialogClose = () => setShowDialog(undefined);
  const [localVariable, setLocalVariable] = useState<WorkflowVariable | null>(
    null,
  );
  const [hasChanges, setHasChanges] = useState(false);

  // Snapshot of the variable when this edit session opened — used to revert on cancel.
  const openedVariableRef = useRef<WorkflowVariable | null>(null);

  const handleAssetDoubleClick = (asset: Asset) => {
    if (localVariable && variable) {
      const updated = { ...localVariable, defaultValue: asset.url };
      setLocalVariable(updated);
      setHasChanges(true);
      onLiveUpdate?.(updated);
    }
    setAssetUrl(asset.url);
    handleDialogClose();
  };

  const handleCmsItemValue = (cmsItemAssetUrl: string) => {
    if (localVariable && variable) {
      const updated = { ...localVariable, defaultValue: cmsItemAssetUrl };
      setLocalVariable(updated);
      setHasChanges(true);
      onLiveUpdate?.(updated);
    }
    setAssetUrl(cmsItemAssetUrl);
    handleDialogClose();
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
    showDialog,
    assetUrl,
    handleAssetDoubleClick,
    handleCmsItemValue,
    handleFieldUpdate,
    handleSave,
    handleCancel,
    handleDialogOpen,
    handleDialogClose,
    clearUrl,
  };
};
