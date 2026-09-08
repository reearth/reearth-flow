-- Create "workflows" table
CREATE TABLE "workflows" (
  "id" text NOT NULL,
  "project_id" text NOT NULL,
  "workspace_id" text NOT NULL,
  "url" text NOT NULL DEFAULT '',
  PRIMARY KEY ("id")
);
-- Create index "workflows_project_id_idx" to table: "workflows"
CREATE INDEX "workflows_project_id_idx" ON "workflows" ("project_id");
-- Create index "workflows_workspace_id_idx" to table: "workflows"
CREATE INDEX "workflows_workspace_id_idx" ON "workflows" ("workspace_id");
