import { useCallback, useState } from "react";
import type { Doc } from "yjs";

import { useProjectExport, useSharedProjectImport } from "@flow/hooks";
import type { Workspace, Project } from "@flow/types";

export default ({
  yDoc,
  project,
  accessToken,
}: {
  yDoc: Doc | null;
  project?: Project;
  accessToken?: string;
}) => {
  const { handleProjectExport } = useProjectExport(project);
  const [selectedWorkspace, setSelectedWorkspace] = useState<Workspace | null>(
    null,
  );
  const { handleProjectImport } = useSharedProjectImport({
    sharedYdoc: yDoc,
    sharedProject: project,
    selectedWorkspace,
    accessToken,
  });
  const [showDialog, setShowDialog] = useState<"import" | undefined>(undefined);

  const handleShowImportDialog = () => setShowDialog("import");

  const handleSelectWorkspace = useCallback((workspace: Workspace | null) => {
    setSelectedWorkspace(workspace);
  }, []);

  const handleDialogClose = useCallback(() => {
    setShowDialog(undefined);
    handleSelectWorkspace(null);
  }, [handleSelectWorkspace]);

  return {
    showDialog,
    selectedWorkspace,
    handleProjectExport,
    handleProjectImport,
    handleShowImportDialog,
    handleSelectWorkspace,
    handleDialogClose,
  };
};
