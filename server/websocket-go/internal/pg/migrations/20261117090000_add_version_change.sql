-- Create "ws_version_change" table
CREATE UNLOGGED TABLE "ws_version_change" (
  "doc_id" text NOT NULL,
  "contributors" text[] NOT NULL DEFAULT '{}'::text[],
  "last_editor" text NOT NULL DEFAULT '',
  "editor_at" timestamptz NULL,
  "first_at" timestamptz NOT NULL,
  "last_at" timestamptz NOT NULL,
  PRIMARY KEY ("doc_id")
);
-- Create index "ws_version_change_last_at_idx" to table: "ws_version_change"
CREATE INDEX "ws_version_change_last_at_idx" ON "ws_version_change" ("last_at");
-- Set comment to column: "contributors" on table: "ws_version_change"
COMMENT ON COLUMN "ws_version_change"."contributors" IS 'users who edited, sorted';
-- Set comment to column: "last_editor" on table: "ws_version_change"
COMMENT ON COLUMN "ws_version_change"."last_editor" IS 'user who sent the latest edit that named its sender';
