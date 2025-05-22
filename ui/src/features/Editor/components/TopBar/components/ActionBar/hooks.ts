import { useState } from "react";

import { useProjectExport } from "@flow/hooks";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const [currentProject] = useCurrentProject();

  const { handleProjectExport } = useProjectExport(currentProject);

  const [showDialog, setShowDialog] = useState<
    "deploy" | "share" | "version" | "debugStop" | undefined
  >(undefined);

  const handleShowDeployDialog = () => setShowDialog("deploy");
  const handleShowShareDialog = () => setShowDialog("share");
  const handleShowVersionDialog = () => setShowDialog("version");
  const handleDialogClose = () => setShowDialog(undefined);

  return {
    showDialog,
    handleShowDeployDialog,
    handleShowShareDialog,
    handleShowVersionDialog,
    handleDialogClose,
    handleProjectExport,
  };
};
