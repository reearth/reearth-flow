import type * as Y from "yjs";

import type { Direction } from "./layout";

// What the editor knows about the project's deployment. Written only by the
// editor, so a deployment made elsewhere leaves a stale version here.
export type DeploymentTracking = {
  // The deployment version this record describes, e.g. "v3".
  version: string;
  // Fingerprint of the deployed graph, compared on undo/redo.
  fingerprint: string;
  // Set on the first relevant edit after deploying; cleared by undo or redeploy.
  changed: boolean;
};

export type YDocMetadata = {
  initialized?: boolean;
  rollbackInProgress?: boolean;
  isLocked?: boolean;
  sharingToken?: string | null;
  layoutDirection?: Direction;
  layoutApplyToAll?: boolean;
  deploymentTracking?: DeploymentTracking;
};

export type YDocMetadataValue = NonNullable<
  YDocMetadata[keyof YDocMetadata]
> | null;

export type YDocMetadataMap = Y.Map<YDocMetadataValue>;
