import { useCallback, useEffect, useMemo, useState } from "react";
import { useY } from "react-yjs";
import type { Doc } from "yjs";
import { Map as YMap } from "yjs";

import { useProject } from "@flow/lib/gql";
import {
  isNewerThanProject,
  readProjectLock,
  resolveProjectLock,
} from "@flow/lib/yjs/projectLock";
import type { Project, YDocMetadataValue } from "@flow/types";

const emptyMetadata = new YMap<YDocMetadataValue>();

export default ({
  currentProject,
  yDoc,
}: {
  currentProject?: Project | undefined;
  yDoc: Doc | null;
}) => {
  const { updateProject, refetchProject } = useProject();

  const yMetadata = useMemo(
    () => yDoc?.getMap<YDocMetadataValue>("metadata"),
    [yDoc],
  );
  const metadata = useY(yMetadata ?? emptyMetadata);
  const signal = readProjectLock(metadata?.projectLock);

  // The lock this user asked for, shown until the project reflects it.
  const [requested, setRequested] = useState<boolean>();

  const resolved = resolveProjectLock(currentProject, signal);
  const isLocked = requested ?? resolved;

  useEffect(() => {
    if (requested !== undefined && resolved === requested) {
      setRequested(undefined);
    }
  }, [requested, resolved]);

  // Another user changed the lock: fetch the project so it catches up.
  const projectId = currentProject?.id;
  const signalIsNewer = isNewerThanProject(signal, currentProject);
  useEffect(() => {
    if (signalIsNewer && projectId) refetchProject(projectId);
  }, [signalIsNewer, projectId, refetchProject]);

  const handleLockChange = useCallback(
    async (lock: boolean) => {
      if (!projectId || requested !== undefined) return;
      setRequested(lock);
      const { project } = await updateProject({ projectId, isLocked: lock });
      if (!project) setRequested(undefined);
    },
    [projectId, requested, updateProject],
  );

  return {
    isLocked,
    handleProjectLockChange: handleLockChange,
  };
};
