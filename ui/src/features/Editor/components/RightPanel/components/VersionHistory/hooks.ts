import { useCallback, useState } from "react";
import { Doc } from "yjs";
import * as Y from "yjs";

import { useDocument } from "@flow/lib/gql/document/useApi";
import { YWorkflow } from "@flow/lib/yjs/types";

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
      const tempWorkflows = tempYDoc.getMap<YWorkflow>("workflows");
      if (!tempWorkflows) {
        console.warn("⚠️ No workflows found inside the rollback update.");
      } else {
        console.log(
          "Workflows inside rollback update:",
          tempWorkflows.toJSON(),
        );
      }

      yDoc.transact(() => {
        const yWorkflows = yDoc.getMap<YWorkflow>("workflows");

        if (yWorkflows) {
          console.log("Deleting existing workflows");
          yWorkflows.clear();
        }

        console.log("Inserting rollback workflows");
        tempWorkflows.forEach((yWorkflow, wId) => {
          yWorkflows.set(wId, yWorkflow);
        });

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
