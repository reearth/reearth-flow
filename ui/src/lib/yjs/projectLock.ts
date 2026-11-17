import type * as Y from "yjs";

import type { YDocMetadataMap, YDocMetadataValue } from "@flow/types";

// The metadata entry the websocket server writes when the project is locked or
// unlocked. Any client with write access could write it too, and only the
// server can check its signature, so the UI never takes the lock from it: a
// change is a cue to fetch the Project, which holds the lock.
export const PROJECT_LOCK_KEY = "projectLock";

// A comparable form of the entry, so a change can be told from the same entry
// written again.
export const projectLockEntryKey = (value: unknown): string | undefined =>
  value === undefined ? undefined : JSON.stringify(value);

// Whether the entry claims a lock. Read only to stop editing until the
// Project is fetched again; a false claim costs a moment of read-only.
export const claimsLock = (value: unknown): boolean =>
  !!value &&
  typeof value === "object" &&
  (value as { locked?: unknown }).locked === true;

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
  let last = projectLockEntryKey(metadata.get(PROJECT_LOCK_KEY));

  const handler = (event: Y.YMapEvent<YDocMetadataValue>) => {
    if (!event.changes.keys.has(PROJECT_LOCK_KEY)) return;
    const current = projectLockEntryKey(metadata.get(PROJECT_LOCK_KEY));
    // While syncing, the entry is the state the connection was opened with.
    if (!provider.synced || current === last) {
      last = current;
      return;
    }
    last = current;
    provider.disconnect();
    provider.connect();
  };

  metadata.observe(handler);
  return () => metadata.unobserve(handler);
};
