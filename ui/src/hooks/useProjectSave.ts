import { useCallback } from "react";

import { useDocument } from "@flow/lib/gql/document";
import { useIsSaving } from "@flow/stores";

export default ({ projectId }: { projectId?: string }) => {
  const { useSaveSnapshot } = useDocument();
  const [isSaving, setIsSaving] = useIsSaving();
  const handleProjectSnapshotSave = useCallback(async () => {
    try {
      setIsSaving(true);
      if (!projectId) {
        console.error("Project ID is required to save a snapshot.");
        setIsSaving(false);
        return;
      }
      await useSaveSnapshot(projectId);
      setIsSaving(false);
    } catch (error) {
      console.error("Error saving project snapshot:", error);
      setIsSaving(false);
    }
  }, [projectId, useSaveSnapshot, setIsSaving]);

  return {
    handleProjectSnapshotSave,
    isSaving,
  };
};
