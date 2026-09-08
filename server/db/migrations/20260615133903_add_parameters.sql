-- Create "parameters" table
CREATE TABLE "parameters" (
  "id" text NOT NULL,
  "project_id" text NOT NULL,
  "name" text NOT NULL,
  "type" text NOT NULL,
  "index" integer NOT NULL DEFAULT 0,
  "required" boolean NOT NULL DEFAULT false,
  "public" boolean NOT NULL DEFAULT false,
  "default_value" jsonb NULL,
  "config" jsonb NULL,
  "created_at" timestamptz NOT NULL,
  "updated_at" timestamptz NOT NULL,
  PRIMARY KEY ("id")
);
-- Create index "parameters_project_id_idx" to table: "parameters"
CREATE INDEX "parameters_project_id_idx" ON "parameters" ("project_id");
