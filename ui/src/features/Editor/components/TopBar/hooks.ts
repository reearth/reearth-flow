import { useState } from "react";

export type DialogOptions =
  | "deploy"
  | "share"
  | "version"
  | "assets"
  | "debugStop"
  | undefined;
export default () => {
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);

  const handleShowDeployDialog = () => setShowDialog("deploy");

  const handleShowVersionDialog = () => setShowDialog("version");

  const handleShowAssetsDialog = () => setShowDialog("assets");

  const handleShowSharePopover = () => setShowDialog("share");

  const handleDialogClose = () => setShowDialog(undefined);
  return {
    showDialog,
    handleShowDeployDialog,
    handleShowVersionDialog,
    handleShowAssetsDialog,
    handleShowSharePopover,
    handleDialogClose,
  };
};
