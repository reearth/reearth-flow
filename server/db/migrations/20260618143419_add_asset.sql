-- Create "asset_uploads" table
CREATE TABLE "asset_uploads" (
  "uuid" text NOT NULL,
  "workspace_id" text NOT NULL,
  "file_name" text NOT NULL DEFAULT '',
  "content_type" text NOT NULL DEFAULT '',
  "content_encoding" text NOT NULL DEFAULT '',
  "content_length" bigint NOT NULL DEFAULT 0,
  "expires_at" timestamptz NOT NULL,
  PRIMARY KEY ("uuid")
);
-- Create index "asset_uploads_workspace_id_idx" to table: "asset_uploads"
CREATE INDEX "asset_uploads_workspace_id_idx" ON "asset_uploads" ("workspace_id");
-- Create "assets" table
CREATE TABLE "assets" (
  "id" text NOT NULL,
  "workspace_id" text NOT NULL,
  "created_at" timestamptz NOT NULL,
  "name" text NOT NULL DEFAULT '',
  "file_name" text NOT NULL DEFAULT '',
  "size" bigint NOT NULL DEFAULT 0,
  "url" text NOT NULL DEFAULT '',
  "content_type" text NOT NULL DEFAULT '',
  "uuid" text NOT NULL DEFAULT '',
  "flat_files" boolean NOT NULL DEFAULT false,
  "public" boolean NOT NULL DEFAULT false,
  "project_id" text NULL,
  "user_id" text NULL,
  "integration_id" text NULL,
  "thread_id" text NULL,
  "archive_extraction_status" text NULL,
  PRIMARY KEY ("id")
);
-- Create index "assets_workspace_id_idx" to table: "assets"
CREATE INDEX "assets_workspace_id_idx" ON "assets" ("workspace_id");
