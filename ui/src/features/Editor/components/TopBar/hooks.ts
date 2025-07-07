import { useState } from "react";

export type DialogOptions =
  | "deploy"
  | "share"
  | "version"
  | "assets"
  | "debugStop"
  | "projectVariables"
  | undefined;
export default () => {
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const handleDialogOpen = (dialog: DialogOptions) => setShowDialog(dialog);
  const handleDialogClose = () => setShowDialog(undefined);
  return {
    showDialog,
    handleDialogOpen,
    handleDialogClose,
  };
};
