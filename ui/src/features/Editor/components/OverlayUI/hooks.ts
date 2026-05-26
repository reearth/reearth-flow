import { useState } from "react";

import { DialogOptions } from "./types";

export default ({
  onUserFocusedElement,
}: {
  onUserFocusedElement?: (isOpen: boolean) => void;
}) => {
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const handleDialogOpen = (dialog: DialogOptions) => {
    setShowDialog(dialog);
    onUserFocusedElement?.(true);
  };
  const handleDialogClose = () => {
    setShowDialog(undefined);
    onUserFocusedElement?.(false);
  };

  return {
    showDialog,
    handleDialogOpen,
    handleDialogClose,
  };
};
