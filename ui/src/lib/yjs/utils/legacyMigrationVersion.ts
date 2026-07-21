import type * as Y from "yjs";

import type { YDocMetadataValue } from "@flow/types";

// Bump when a new legacy-data migration ships: docs stamped with an older
// version get scanned (and prompted) once more, then re-stamped.
// 1 = legacy "default" ports + pre-rename action names (engine PRs #2236/#2240).
export const CURRENT_LEGACY_MIGRATION_VERSION = 1;

const getMetadata = (yDoc?: Y.Doc | null) =>
  yDoc?.getMap<YDocMetadataValue>("metadata");

/**
 * True when the doc is stamped as already migrated (or was created after the
 * renames), so the per-change legacy-data scans can be skipped entirely.
 */
export const isLegacyMigrationComplete = (yDoc?: Y.Doc | null): boolean => {
  const version = getMetadata(yDoc)?.get("legacyMigrationVersion");
  return (
    typeof version === "number" && version >= CURRENT_LEGACY_MIGRATION_VERSION
  );
};

/**
 * Stamps the doc as migrated. Call inside the same transaction as the
 * migration itself (or when constructing a doc that starts out current), so
 * the stamp and the data it vouches for always sync together.
 */
export const markLegacyMigrationComplete = (yDoc?: Y.Doc | null): void => {
  getMetadata(yDoc)?.set(
    "legacyMigrationVersion",
    CURRENT_LEGACY_MIGRATION_VERSION,
  );
};
