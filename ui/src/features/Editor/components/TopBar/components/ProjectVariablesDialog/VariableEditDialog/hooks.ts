import { useState, useEffect, useCallback } from "react";

import { Asset, ProjectVariable } from "@flow/types";

export type DialogOptions = "assets" | "cms" | undefined;
export default ({
  variable,
  onClose,
  onUpdate,
}: {
  variable: ProjectVariable | null;
  onClose: () => void;
  onUpdate: (variable: ProjectVariable) => void;
}) => {
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const [assetUrl, setAssetUrl] = useState<string | null>(null);
  const handleDialogOpen = (dialog: DialogOptions) => setShowDialog(dialog);
  const handleDialogClose = () => setShowDialog(undefined);
  const [localVariable, setLocalVariable] = useState<ProjectVariable | null>(
    null,
  );
  const [hasChanges, setHasChanges] = useState(false);
  const handleAssetDoubleClick = (asset: Asset) => {
    if (localVariable && variable) {
      setLocalVariable({ ...localVariable, defaultValue: asset.url });
      setHasChanges(true);
    }
    setAssetUrl(asset.url);
    handleDialogClose();
  };

  const handleCmsItemValue = (cmsItemAssetUrl: string) => {
    if (localVariable && variable) {
      setLocalVariable({ ...localVariable, defaultValue: cmsItemAssetUrl });
      setHasChanges(true);
    }
    setAssetUrl(cmsItemAssetUrl);
    handleDialogClose();
  };

  useEffect(() => {
    if (variable) {
      setLocalVariable({ ...variable });
      setHasChanges(false);
    } else {
      setLocalVariable(null);
      setHasChanges(false);
    }
  }, [variable]);

  const handleFieldUpdate = useCallback((updatedVariable: ProjectVariable) => {
    setLocalVariable(updatedVariable);
    setHasChanges(true);
  }, []);

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
