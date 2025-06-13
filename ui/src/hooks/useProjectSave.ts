import { useCallback } from "react";

import { useDocument } from "@flow/lib/gql/document";

export default ({ projectId }: { projectId?: string }) => {
  const { useSaveSnapshot } = useDocument();

  const handleProjectSnapshotSave = useCallback(async () => {
    try {
      if (!projectId) {
        console.error("Project ID is required to save a snapshot.");
        return;
      }
      await useSaveSnapshot(projectId);
    } catch (error) {
      console.error("Error saving project snapshot:", error);
    }
  }, [projectId, useSaveSnapshot]);

  return {
    handleProjectSnapshotSave,
  };
};
