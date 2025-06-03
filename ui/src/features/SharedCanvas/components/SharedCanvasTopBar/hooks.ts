import { useCallback, useState } from "react";
import type { Doc } from "yjs";

import { useProjectExport } from "@flow/hooks";
import type { Workspace, Project } from "@flow/types";

import useSharedProjectImport from "./useSharedProjectImport";

export default ({
  yDoc,
  project,
  accessToken,
}: {
  yDoc: Doc | null;
  project?: Project;
  accessToken?: string;
}) => {
  const [showDialog, setShowDialog] = useState<"import" | undefined>(undefined);
  const [selectedWorkspace, setSelectedWorkspace] = useState<Workspace | null>(
    null,
  );

  const { handleProjectExport } = useProjectExport(project);

  const { handleSharedProjectImport } = useSharedProjectImport({
    sharedYdoc: yDoc,
    sharedProject: project,
    selectedWorkspace,
    accessToken,
  });

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
    handleSharedProjectImport,
    handleShowImportDialog,
    handleSelectWorkspace,
    handleDialogClose,
  };
};
