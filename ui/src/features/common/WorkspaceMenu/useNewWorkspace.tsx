import { useState } from "react";

import { dashboardNewWorkspaceUrl, openLinkInNewTab } from "@flow/utils";

import { WorkspaceAddDialog } from "./WorkspaceAddDialog";

export const useNewWorkspace = () => {
  const [openWorkspaceAddDialog, setOpenWorkspaceAddDialog] = useState(false);

  const handleNewWorkspace = () => {
    const externalNewWorkspaceUrl = dashboardNewWorkspaceUrl();
    if (externalNewWorkspaceUrl) {
      openLinkInNewTab(externalNewWorkspaceUrl)();
      return;
    }
    setOpenWorkspaceAddDialog(true);
  };

  const workspaceAddDialog = openWorkspaceAddDialog && (
    <WorkspaceAddDialog
      isOpen={openWorkspaceAddDialog}
      onOpenChange={setOpenWorkspaceAddDialog}
    />
  );

  return { handleNewWorkspace, workspaceAddDialog };
};
