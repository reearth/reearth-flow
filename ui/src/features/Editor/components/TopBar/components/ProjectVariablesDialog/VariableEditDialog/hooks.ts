import { useState, useEffect, useCallback } from "react";

import { ProjectVariable } from "@flow/types";

export default ({
  variable,
  onClose,
  onUpdate,
}: {
  variable: ProjectVariable | null;
  onClose: () => void;
  onUpdate: (variable: ProjectVariable) => void;
}) => {
  const [localVariable, setLocalVariable] = useState<ProjectVariable | null>(
    null,
  );
  const [hasChanges, setHasChanges] = useState(false);

  // Initialize local state when variable changes
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
      console.log("Saving variable:", localVariable);
      onUpdate(localVariable);
    }
    onClose();
  }, [localVariable, hasChanges, onUpdate, onClose]);

  const handleCancel = useCallback(() => {
    setHasChanges(false);
    onClose();
  }, [onClose]);

  return {
    localVariable,
    hasChanges,
    handleFieldUpdate,
    handleSave,
    handleCancel,
  };
};
