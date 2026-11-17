-- Create "ws_version_change" table
CREATE UNLOGGED TABLE "ws_version_change" (
  "id" bigserial NOT NULL,
  "doc_id" text NOT NULL,
  "first_at" timestamptz NOT NULL,
  "batch" bytea NOT NULL,
  PRIMARY KEY ("id")
);
-- Create index "ws_version_change_doc_id_idx" to table: "ws_version_change"
CREATE INDEX "ws_version_change_doc_id_idx" ON "ws_version_change" ("doc_id");
-- Create index "ws_version_change_first_at_idx" to table: "ws_version_change"
CREATE INDEX "ws_version_change_first_at_idx" ON "ws_version_change" ("first_at");
-- Set comment to column: "first_at" on table: "ws_version_change"
COMMENT ON COLUMN "ws_version_change"."first_at" IS 'the batch''s oldest edit';
-- Set comment to column: "batch" on table: "ws_version_change"
COMMENT ON COLUMN "ws_version_change"."batch" IS 'the batch''s edits by user (versions.EncodeRecord)';
