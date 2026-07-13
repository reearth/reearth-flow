import { describe, expect, it } from "vitest";
import * as Y from "yjs";

import {
  CURRENT_LEGACY_MIGRATION_VERSION,
  isLegacyMigrationComplete,
  markLegacyMigrationComplete,
} from "./legacyMigrationVersion";

describe("legacyMigrationVersion", () => {
  it("treats an unstamped doc as not migrated", () => {
    expect(isLegacyMigrationComplete(new Y.Doc())).toBe(false);
    expect(isLegacyMigrationComplete(null)).toBe(false);
    expect(isLegacyMigrationComplete(undefined)).toBe(false);
  });

  it("marks and detects a migrated doc", () => {
    const yDoc = new Y.Doc();
    markLegacyMigrationComplete(yDoc);
    expect(isLegacyMigrationComplete(yDoc)).toBe(true);
    expect(yDoc.getMap("metadata").get("legacyMigrationVersion")).toBe(
      CURRENT_LEGACY_MIGRATION_VERSION,
    );
  });

  it("treats a stamp from an older migration round as not migrated", () => {
    const yDoc = new Y.Doc();
    yDoc
      .getMap("metadata")
      .set("legacyMigrationVersion", CURRENT_LEGACY_MIGRATION_VERSION - 1);
    expect(isLegacyMigrationComplete(yDoc)).toBe(false);
  });

  it("keeps a stamp from a newer client valid", () => {
    const yDoc = new Y.Doc();
    yDoc
      .getMap("metadata")
      .set("legacyMigrationVersion", CURRENT_LEGACY_MIGRATION_VERSION + 1);
    expect(isLegacyMigrationComplete(yDoc)).toBe(true);
  });
});
