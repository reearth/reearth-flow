import { useState } from "react";

export default () => {
  const [showDialog, setShowDialog] = useState<
    "deploy" | "share" | "version" | "debugStop" | undefined
  >(undefined);

  const handleShowDeployDialog = () => setShowDialog("deploy");

  const handleShowVersionDialog = () => setShowDialog("version");

  const handleShowSharePopover = () => setShowDialog("share");

  const handleDialogClose = () => setShowDialog(undefined);

  return {
    showDialog,
    handleShowDeployDialog,
    handleShowVersionDialog,
    handleShowSharePopover,
    handleDialogClose,
  };
};
