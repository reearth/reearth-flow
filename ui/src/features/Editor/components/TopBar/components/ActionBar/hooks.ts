import { useState } from "react";

import { useProjectExport } from "@flow/hooks";

export default () => {
  const { handleProjectExport } = useProjectExport();

  const [showDialog, setShowDialog] = useState<
    "deploy" | "share" | "debugStop" | undefined
  >(undefined);

  const handleShowDeployDialog = () => setShowDialog("deploy");
  const handleShowSharePopover = () => setShowDialog("share");
  const handleDialogClose = () => setShowDialog(undefined);

  return {
    showDialog,
    handleShowDeployDialog,
    handleShowSharePopover,
    handleDialogClose,
    handleProjectExport,
  };
};
