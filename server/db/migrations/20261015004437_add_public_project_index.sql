-- Create index "projects_visibility_is_archived_updated_at_idx" to table: "projects"
CREATE INDEX "projects_visibility_is_archived_updated_at_idx" ON "projects" ("visibility", "is_archived", "updated_at" DESC, "id" DESC);
