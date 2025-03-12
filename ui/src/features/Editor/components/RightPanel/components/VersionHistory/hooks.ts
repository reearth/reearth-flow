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
      const rollbackData = await useRollbackProject(
        projectId,
        selectedProjectSnapshotVersion,
      );
      const updates = rollbackData.projectDocument?.updates;

      if (!updates || !updates.length || !yDoc) {
        return;
      }

      const convertedUpdates = new Uint8Array(updates);

      console.log("Update contents...");

      // Load temp doc to check if workflows are present
      const tempYDoc = new Y.Doc();
      Y.applyUpdate(tempYDoc, convertedUpdates);
      // for testing but could use temp doc to convert etc
      const tempWorkflows = tempYDoc.getArray("workflows");
      if (!tempWorkflows.length) {
        console.warn("⚠️ No workflows found inside the rollback update.");
      } else {
        console.log(
          "Workflows inside rollback update:",
          tempWorkflows.toJSON(),
        );
      }

      yDoc.transact(() => {
        const yWorkflows = yDoc.getArray("workflows");

        if (yWorkflows.length) {
          console.log("Deleting existing workflows");
          yWorkflows.delete(0, yWorkflows.length);
        }

        // Fails here possibly due to a null value
        // Insert rollback workflows
        console.log("Inserting rollback workflows");
        yWorkflows.insert(0, tempWorkflows.toArray());

        console.log(
          "Workflows inside yDoc after rollback:",
          yWorkflows.toJSON(),
        );
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
