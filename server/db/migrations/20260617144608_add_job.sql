-- Create "jobs" table
CREATE TABLE "jobs" (
  "id" text NOT NULL,
  "workspace_id" text NOT NULL,
  "deployment_id" text NULL,
  "project_id" text NULL,
  "project_version" integer NULL,
  "gcp_job_id" text NOT NULL DEFAULT '',
  "logs_url" text NOT NULL DEFAULT '',
  "worker_logs_url" text NOT NULL DEFAULT '',
  "user_facing_logs_url" text NOT NULL DEFAULT '',
  "status" text NOT NULL DEFAULT '',
  "batch_status" text NULL,
  "worker_status" text NULL,
  "started_at" timestamptz NOT NULL,
  "completed_at" timestamptz NULL,
  "metadata_url" text NOT NULL DEFAULT '',
  "output_urls" jsonb NULL,
  "debug" boolean NULL,
  "parameters" jsonb NULL,
  PRIMARY KEY ("id")
);
-- Create index "jobs_deployment_id_idx" to table: "jobs"
CREATE INDEX "jobs_deployment_id_idx" ON "jobs" ("deployment_id");
-- Create index "jobs_project_id_idx" to table: "jobs"
CREATE INDEX "jobs_project_id_idx" ON "jobs" ("project_id");
-- Create index "jobs_workspace_id_debug_idx" to table: "jobs"
CREATE INDEX "jobs_workspace_id_debug_idx" ON "jobs" ("workspace_id", "debug");
-- Create index "jobs_workspace_id_idx" to table: "jobs"
CREATE INDEX "jobs_workspace_id_idx" ON "jobs" ("workspace_id");
