-- Create "project_accesses" table
CREATE TABLE "project_accesses" (
  "id" text NOT NULL,
  "project_id" text NOT NULL,
  "token" text NOT NULL DEFAULT '',
  "is_public" boolean NOT NULL DEFAULT false,
  PRIMARY KEY ("id")
);
-- Create index "project_accesses_project_id_idx" to table: "project_accesses"
CREATE UNIQUE INDEX "project_accesses_project_id_idx" ON "project_accesses" ("project_id");
-- Create index "project_accesses_token_idx" to table: "project_accesses"
CREATE INDEX "project_accesses_token_idx" ON "project_accesses" ("token");
