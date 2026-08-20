-- Create "edge_executions" table
CREATE TABLE "edge_executions" (
  "id" text NOT NULL,
  "edge_id" text NOT NULL,
  "job_id" text NOT NULL,
  "intermediate_data_url" text NULL,
  PRIMARY KEY ("id")
);
-- Create index "edge_executions_job_id_edge_id_idx" to table: "edge_executions"
CREATE INDEX "edge_executions_job_id_edge_id_idx" ON "edge_executions" ("job_id", "edge_id");
-- Create index "edge_executions_job_id_idx" to table: "edge_executions"
CREATE INDEX "edge_executions_job_id_idx" ON "edge_executions" ("job_id");
