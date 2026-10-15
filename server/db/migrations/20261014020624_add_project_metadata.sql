-- Modify "projects" table
ALTER TABLE "projects" ADD COLUMN "alias" text NOT NULL DEFAULT '', ADD COLUMN "visibility" text NOT NULL DEFAULT 'private', ADD COLUMN "readme" text NOT NULL DEFAULT '', ADD COLUMN "license" text NOT NULL DEFAULT '', ADD COLUMN "topics" jsonb NULL, ADD COLUMN "starred_by" jsonb NULL;
-- Give existing projects their default alias (see project.DefaultAlias) so the unique index below can be built
UPDATE "projects" SET "alias" = 'p-' || "id";
-- Every write sets the alias, so the column keeps no default
ALTER TABLE "projects" ALTER COLUMN "alias" DROP DEFAULT;
-- Create index "projects_workspace_id_alias_idx" to table: "projects"
CREATE UNIQUE INDEX "projects_workspace_id_alias_idx" ON "projects" ("workspace_id", "alias");
