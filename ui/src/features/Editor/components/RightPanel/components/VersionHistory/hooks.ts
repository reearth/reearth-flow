import { useCallback, useState } from "react";
import { Doc } from "yjs";
import * as Y from "yjs";

import { useDocument } from "@flow/lib/gql/document/useApi";

export default ({
  projectId,
  yDoc,
}: {
  projectId: string;
  yDoc: Doc | null;
}) => {
  const {
    useGetProjectHistory,
    useGetLatestProjectSnapshot,
    useRollbackProject,
  } = useDocument();

  const { history, isFetching } = useGetProjectHistory(projectId);

  const { projectDocument } = useGetLatestProjectSnapshot(projectId);

  const [selectedProjectSnapshotVersion, setSelectedProjectSnapshotVersion] =
    useState<number | null>(null);
  const [openVersionChangeDialog, setOpenVersionChangeDialog] =
    useState<boolean>(false);

  const handleRollbackProject = useCallback(async () => {
    if (selectedProjectSnapshotVersion === null) return;

    try {
      console.log(
        "Starting rollback to version:",
        selectedProjectSnapshotVersion,
      );

      const rollbackData = await useRollbackProject(
        projectId,
        selectedProjectSnapshotVersion,
      );
      const updates = rollbackData.projectDocument?.updates;

      if (!updates || !updates.length) {
        console.error("No updates found for rollback version.");
        return;
      }

      console.log(
        "Retrieved rollback snapshot with",
        updates.length,
        "updates.",
      );

      if (!yDoc) {
        console.error("No existing Y.Doc found.");
        return;
      }

      yDoc.transact(() => {
        const emptyState = Y.encodeStateAsUpdate(new Y.Doc());
        Y.applyUpdate(yDoc, emptyState);
      });

      console.log("⚠️ Y.Doc cleared. Applying rollback updates...");

      const convertedUpdates = new Uint8Array(updates);

      console.log("Rollback updates applied successfully.");

      const yWorkflows = yDoc.getArray("workflows");

      if (!yWorkflows.length) {
        console.warn("⚠️ No workflows found after rollback.");
      } else {
        console.log(
          "Workflows successfully restored after rollback:",
          yWorkflows.toJSON(),
        );
      }

      yDoc.transact(() => {
        Y.applyUpdate(yDoc, convertedUpdates);
      });

      console.log("Rollback completed successfully.");
    } catch (error) {
      console.error("Project Rollback Failed:", error);
    }

    setOpenVersionChangeDialog(false);
  }, [selectedProjectSnapshotVersion, useRollbackProject, projectId, yDoc]);

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
