import { useState } from "react";

import { DialogOptions } from "./types";

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
