-- Create "deployments" table
CREATE TABLE "deployments" (
  "id" text NOT NULL,
  "workspace_id" text NOT NULL,
  "project_id" text NULL,
  "workflow_url" text NOT NULL DEFAULT '',
  "description" text NOT NULL DEFAULT '',
  "version" text NOT NULL DEFAULT '',
  "updated_at" timestamptz NOT NULL,
  "head_id" text NULL,
  "is_head" boolean NOT NULL DEFAULT false,
  PRIMARY KEY ("id")
);
-- Create index "deployments_project_id_idx" to table: "deployments"
CREATE INDEX "deployments_project_id_idx" ON "deployments" ("project_id");
-- Create index "deployments_workspace_id_idx" to table: "deployments"
CREATE INDEX "deployments_workspace_id_idx" ON "deployments" ("workspace_id");
-- Create index "deployments_workspace_id_is_head_idx" to table: "deployments"
CREATE INDEX "deployments_workspace_id_is_head_idx" ON "deployments" ("workspace_id", "is_head");
