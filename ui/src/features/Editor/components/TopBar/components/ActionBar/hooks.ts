import { useState } from "react";

import { useProjectSave } from "@flow/hooks";

export default ({ projectId }: { projectId: string }) => {
  const [showDialog, setShowDialog] = useState<
    "deploy" | "share" | "version" | "debugStop" | undefined
  >(undefined);
  const { handleProjectSnapshotSave } = useProjectSave({ projectId });

  const handleShowDeployDialog = () => setShowDialog("deploy");

  const handleShowVersionDialog = () => setShowDialog("version");

  const handleShowSharePopover = () => setShowDialog("share");

  const handleDialogClose = () => setShowDialog(undefined);
  return {
    showDialog,
    handleProjectSnapshotSave,
    handleShowDeployDialog,
    handleShowVersionDialog,
    handleShowSharePopover,
    handleDialogClose,
  };
};
