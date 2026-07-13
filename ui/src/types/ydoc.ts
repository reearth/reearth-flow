import type * as Y from "yjs";

import type { Direction } from "./layout";

export type YDocMetadata = {
  initialized?: boolean;
  rollbackInProgress?: boolean;
  legacyMigrationVersion?: number;
  isLocked?: boolean;
  sharingToken?: string | null;
  layoutDirection?: Direction;
  layoutApplyToAll?: boolean;
};

export type YDocMetadataValue = NonNullable<
  YDocMetadata[keyof YDocMetadata]
> | null;

export type YDocMetadataMap = Y.Map<YDocMetadataValue>;
