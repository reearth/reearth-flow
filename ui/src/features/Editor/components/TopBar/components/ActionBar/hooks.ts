import { useState } from "react";

export default () => {
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
  };
};
