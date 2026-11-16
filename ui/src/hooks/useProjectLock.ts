import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useY } from "react-yjs";
import type { Doc } from "yjs";
import { Map as YMap } from "yjs";

import { useProject } from "@flow/lib/gql";
import {
  claimsLock,
  PROJECT_LOCK_KEY,
  projectLockEntryKey,
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
  const entry = metadata?.[PROJECT_LOCK_KEY];
  const entryKey = projectLockEntryKey(entry);

  // The lock this user asked for, shown until the project reflects it.
  const [requested, setRequested] = useState<boolean>();
  // Set while the project is fetched again after the entry claimed a lock.
  const [claimedLock, setClaimedLock] = useState(false);

  const projectLocked = !!currentProject?.isLocked;
  const isLocked = requested ?? (claimedLock || projectLocked);

  useEffect(() => {
    if (requested !== undefined && projectLocked === requested) {
      setRequested(undefined);
    }
  }, [requested, projectLocked]);

  // The server changed the entry: fetch the project, which holds the lock.
  // Until it arrives, a claimed lock stops editing, since the server would
  // refuse the edits anyway.
  const projectId = currentProject?.id;
  const lastEntryKey = useRef(entryKey);
  const fetches = useRef(0);
  useEffect(() => {
    if (entryKey === lastEntryKey.current || !projectId) return;
    lastEntryKey.current = entryKey;
    const fetch = ++fetches.current;
    setClaimedLock(claimsLock(entry));
    Promise.resolve(refetchProject(projectId)).finally(() => {
      if (fetch === fetches.current) setClaimedLock(false);
    });
  }, [entry, entryKey, projectId, refetchProject]);

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
