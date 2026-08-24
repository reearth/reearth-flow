-- Create index "assets_workspace_id_created_at_idx" to table: "assets"
CREATE INDEX "assets_workspace_id_created_at_idx" ON "public"."assets" ("workspace_id", "created_at" DESC);
-- Create index "deployments_workspace_id_updated_at_idx" to table: "deployments"
CREATE INDEX "deployments_workspace_id_updated_at_idx" ON "public"."deployments" ("workspace_id", "updated_at" DESC);
-- Create index "jobs_workspace_id_started_at_idx" to table: "jobs"
CREATE INDEX "jobs_workspace_id_started_at_idx" ON "public"."jobs" ("workspace_id", "started_at" DESC);
-- Create index "projects_workspace_id_is_archived_updated_at_idx" to table: "projects"
CREATE INDEX "projects_workspace_id_is_archived_updated_at_idx" ON "public"."projects" ("workspace_id", "is_archived", "updated_at" DESC);
-- Create index "triggers_workspace_id_updated_at_idx" to table: "triggers"
CREATE INDEX "triggers_workspace_id_updated_at_idx" ON "public"."triggers" ("workspace_id", "updated_at" DESC);
