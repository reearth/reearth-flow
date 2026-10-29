-- Modify "projects" table
ALTER TABLE "projects" ADD COLUMN "created_by" text NULL, ADD COLUMN "updated_by" text NULL;
-- Create index "projects_workspace_id_created_by_idx" to table: "projects"
CREATE INDEX "projects_workspace_id_created_by_idx" ON "projects" ("workspace_id", "created_by");
