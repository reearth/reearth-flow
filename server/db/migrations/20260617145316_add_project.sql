-- Create "projects" table
CREATE TABLE "projects" (
  "id" text NOT NULL,
  "workspace_id" text NOT NULL,
  "workflow_id" text NOT NULL DEFAULT '',
  "name" text NOT NULL DEFAULT '',
  "description" text NOT NULL DEFAULT '',
  "is_archived" boolean NOT NULL DEFAULT false,
  "is_basic_auth_active" boolean NOT NULL DEFAULT false,
  "basic_auth_username" text NOT NULL DEFAULT '',
  "basic_auth_password" text NOT NULL DEFAULT '',
  "shared_token" text NULL,
  "updated_at" timestamptz NOT NULL,
  "is_locked" boolean NOT NULL DEFAULT false,
  PRIMARY KEY ("id")
);
-- Create index "projects_workspace_id_idx" to table: "projects"
CREATE INDEX "projects_workspace_id_idx" ON "projects" ("workspace_id");
-- Create index "projects_workspace_id_is_archived_idx" to table: "projects"
CREATE INDEX "projects_workspace_id_is_archived_idx" ON "projects" ("workspace_id", "is_archived");
