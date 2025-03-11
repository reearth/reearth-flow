import { useCallback, useState } from "react";

import { useDocument } from "@flow/lib/gql/document/useApi";

export default ({ projectId }: { projectId: string }) => {
  const [openVersionChangeDialog, setOpenVersionChangeDialog] =
    useState<boolean>(false);

  const {
    useGetProjectHistory,
    useGetLatestProjectSnapshot,
    useRollbackProject,
  } = useDocument();

  const { history, isFetching } = useGetProjectHistory(projectId);

  const { projectDocument } = useGetLatestProjectSnapshot(projectId);

  const [selectedProjectSnapshotVersion, setSelectedProjectSnapshotVersion] =
    useState<number | null>(null);

  const handleRollbackProject = useCallback(async () => {
    if (selectedProjectSnapshotVersion === null) return;
    const rollbackData = await useRollbackProject(
      projectId,
      selectedProjectSnapshotVersion,
    );

    console.log("rollbackData", rollbackData);
    setOpenVersionChangeDialog(false);
  }, [selectedProjectSnapshotVersion, useRollbackProject, projectId]);

  const latestProjectSnapshotVersion = projectDocument;
  return {
    history,
    isFetching,
    latestProjectSnapshotVersion,
    selectedProjectSnapshotVersion,
    setSelectedProjectSnapshotVersion,
    openVersionChangeDialog,
    setOpenVersionChangeDialog,
    onRollbackProject: handleRollbackProject,
  };
};
