import { useState } from "react";

import { useProjectSave } from "@flow/hooks";

export default ({ projectId }: { projectId: string }) => {
  const [showDialog, setShowDialog] = useState<
    "deploy" | "share" | "version" | "debugStop" | undefined
  >(undefined);
  const { handleProjectSnapshotSave, isSaving } = useProjectSave({ projectId });

  const handleShowDeployDialog = () => setShowDialog("deploy");

  const handleShowVersionDialog = () => setShowDialog("version");

  const handleShowSharePopover = () => setShowDialog("share");

  const handleDialogClose = () => setShowDialog(undefined);
  return {
    showDialog,
    isSaving,
    handleProjectSnapshotSave,
    handleShowDeployDialog,
    handleShowVersionDialog,
    handleShowSharePopover,
    handleDialogClose,
  };
};
