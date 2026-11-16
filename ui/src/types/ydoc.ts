import type * as Y from "yjs";

import type { Direction } from "./layout";

// The project's last deployment from the editor. Written only on deploy, so a
// deployment made elsewhere leaves a stale version here.
export type DeploymentTracking = {
  // The deployment version this record describes, e.g. "v3".
  version: string;
  // Fingerprint of the deployed graph, compared on undo/redo and on remote
  // changes.
  fingerprint: string;
};

// A project lock change, written into the document by the websocket server
// only. `at` is when it changed, in Unix ms.
export type ProjectLockSignal = {
  locked: boolean;
  at: number;
};

export type YDocMetadata = {
  initialized?: boolean;
  rollbackInProgress?: boolean;
  projectLock?: ProjectLockSignal;
  sharingToken?: string | null;
  layoutDirection?: Direction;
  layoutApplyToAll?: boolean;
  deploymentTracking?: DeploymentTracking;
  // The recorded version the canvas has changed from, or null. A separate key
  // from the record, so an edit and a concurrent deploy cannot overwrite each
  // other.
  deploymentChangedVersion?: string | null;
};

export type YDocMetadataValue = NonNullable<
  YDocMetadata[keyof YDocMetadata]
> | null;

export type YDocMetadataMap = Y.Map<YDocMetadataValue>;
