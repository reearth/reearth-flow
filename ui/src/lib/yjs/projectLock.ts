import type * as Y from "yjs";

import type {
  ProjectLockSignal,
  YDocMetadataMap,
  YDocMetadataValue,
} from "@flow/types";

// The metadata entry the websocket server writes when the project is locked or
// unlocked. Clients never write it: a locked project's connections are
// read-only, so a client could not unlock it through the document anyway.
export const PROJECT_LOCK_KEY = "projectLock";

export const readProjectLock = (
  value: unknown,
): ProjectLockSignal | undefined => {
  if (!value || typeof value !== "object") return undefined;
  const { locked, at } = value as Record<string, unknown>;
  if (typeof locked !== "boolean" || typeof at !== "number") return undefined;
  return { locked, at };
};

// The signal is the lock change the server wrote at `at` (the project's
// updatedAt then, in ms). It is newer than the project only until the project
// is fetched again.
export const isNewerThanProject = (
  signal: ProjectLockSignal | undefined,
  project: { updatedAt?: string } | undefined,
): signal is ProjectLockSignal => {
  if (!signal) return false;
  const updatedAt = project?.updatedAt ? Date.parse(project.updatedAt) : NaN;
  return Number.isNaN(updatedAt) || signal.at > updatedAt;
};

// The project's lock: the Project, unless the server has signalled a newer
// change that the fetched Project does not have yet.
export const resolveProjectLock = (
  project: { isLocked?: boolean; updatedAt?: string } | undefined,
  signal: ProjectLockSignal | undefined,
): boolean =>
  isNewerThanProject(signal, project) ? signal.locked : !!project?.isLocked;

type Reconnectable = {
  synced: boolean;
  connect: () => void;
  disconnect: () => void;
};

// Reconnects the provider when the project is locked or unlocked while it is
// connected, so the server checks the connection's access again. Without this a
// connection opened while the project was locked would stay read-only after an
// unlock, and the server would drop its edits.
export const reconnectOnProjectLockChange = (
  metadata: YDocMetadataMap,
  provider: Reconnectable,
): (() => void) => {
  let lastLocked = readProjectLock(metadata.get(PROJECT_LOCK_KEY))?.locked;

  const handler = (event: Y.YMapEvent<YDocMetadataValue>) => {
    if (!event.changes.keys.has(PROJECT_LOCK_KEY)) return;
    const locked = readProjectLock(metadata.get(PROJECT_LOCK_KEY))?.locked;
    // While syncing, the entry is the state the connection was opened with.
    if (!provider.synced || locked === lastLocked) {
      lastLocked = locked;
      return;
    }
    lastLocked = locked;
    provider.disconnect();
    provider.connect();
  };

  metadata.observe(handler);
  return () => metadata.unobserve(handler);
};
