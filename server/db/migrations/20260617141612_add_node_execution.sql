-- Create "node_executions" table
CREATE TABLE "node_executions" (
  "id" text NOT NULL,
  "job_id" text NOT NULL,
  "node_id" text NOT NULL,
  "status" text NOT NULL DEFAULT 'PENDING',
  "started_at" timestamptz NULL,
  "completed_at" timestamptz NULL,
  PRIMARY KEY ("id")
);
-- Create index "node_executions_job_id_idx" to table: "node_executions"
CREATE INDEX "node_executions_job_id_idx" ON "node_executions" ("job_id");
-- Create index "node_executions_job_id_node_id_idx" to table: "node_executions"
CREATE INDEX "node_executions_job_id_node_id_idx" ON "node_executions" ("job_id", "node_id");
