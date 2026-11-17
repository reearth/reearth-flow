-- An earlier build of this change created the table in another shape under
-- another migration name. It only ever holds unsaved-version markers, so drop
-- it and start again.
DROP TABLE IF EXISTS "ws_version_change";
-- Create "ws_version_change" table
CREATE UNLOGGED TABLE "ws_version_change" (
  "doc_id" text NOT NULL,
  "instance" text NOT NULL,
  "first_at" timestamptz NOT NULL,
  "batch" bytea NOT NULL,
  PRIMARY KEY ("doc_id", "instance")
);
-- Create index "ws_version_change_first_at_idx" to table: "ws_version_change"
CREATE INDEX "ws_version_change_first_at_idx" ON "ws_version_change" ("first_at");
-- Set comment to column: "instance" on table: "ws_version_change"
COMMENT ON COLUMN "ws_version_change"."instance" IS 'the instance that recorded the edits';
-- Set comment to column: "first_at" on table: "ws_version_change"
COMMENT ON COLUMN "ws_version_change"."first_at" IS 'the row''s oldest edit';
-- Set comment to column: "batch" on table: "ws_version_change"
COMMENT ON COLUMN "ws_version_change"."batch" IS 'the row''s edits by user (versions.EncodeRecord)';
