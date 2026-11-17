import { useCallback, useEffect, useMemo } from "react";
import { useY } from "react-yjs";
import type { Doc } from "yjs";
import { Map as YMap } from "yjs";

import { useDebouncedCallback } from "@flow/hooks";
import { useProject } from "@flow/lib/gql";
import type { Project, YDocMetadataValue } from "@flow/types";

const emptyMetadata = new YMap<YDocMetadataValue>();

export default ({
  currentProject,
  yDoc,
}: {
  currentProject?: Project | undefined;
  yDoc: Doc | null;
}) => {
  const { updateProject } = useProject();

  const yMetadata = useMemo(
    () => yDoc?.getMap<YDocMetadataValue>("metadata"),
    [yDoc],
  );
  const metadata = useY(yMetadata ?? emptyMetadata);

  const isLocked: boolean =
    metadata?.isLocked !== undefined
      ? !!metadata.isLocked
      : !!currentProject?.isLocked;

  // Keep ydoc in sync with server state. Every set is a document update, so
  // write only when the doc disagrees; an unconditional write would change the
  // document each time the project is opened. A missing key reads as unlocked.
  useEffect(() => {
    if (!yMetadata || currentProject?.isLocked === undefined) return;
    const serverLocked = !!currentProject.isLocked;
    if (!!yMetadata.get("isLocked") === serverLocked) return;
    yMetadata.set("isLocked", serverLocked);
  }, [yMetadata, currentProject?.isLocked]);

  const debouncedUpdateProject = useDebouncedCallback(async (lock: boolean) => {
    if (!currentProject) return;
    try {
      await updateProject({
        projectId: currentProject.id,
        isLocked: lock,
      });
    } catch (error) {
      yMetadata?.set("isLocked", !lock);
      console.error("Failed to update project lock status:", error);
    }
  }, 2000);

  const handleLockChange = useCallback(
    (lock: boolean) => {
      yMetadata?.set("isLocked", lock);
      debouncedUpdateProject(lock);
    },
    [yMetadata, debouncedUpdateProject],
  );

  return {
    isLocked,
    handleProjectLockChange: handleLockChange,
  };
};
