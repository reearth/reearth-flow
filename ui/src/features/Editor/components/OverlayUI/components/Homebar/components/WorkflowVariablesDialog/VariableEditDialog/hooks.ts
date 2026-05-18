import { useState, useEffect, useCallback } from "react";

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

  // Sync from parent when no local edits in progress (passive viewer update)
  useEffect(() => {
    if (variable) {
      if (!hasChanges) {
        setLocalVariable({ ...variable });
      }
    } else {
      setLocalVariable(null);
      setHasChanges(false);
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
    onClose();
  }, [localVariable, hasChanges, onUpdate, onClose]);

  const handleCancel = useCallback(() => {
    setHasChanges(false);
    onClose();
  }, [onClose]);

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
