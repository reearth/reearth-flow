-- Create "job_diagnostic_summaries" table
CREATE TABLE "job_diagnostic_summaries" (
  "job_id" text NOT NULL,
  "ts" timestamptz NOT NULL,
  "dropped_event_count" bigint NOT NULL,
  PRIMARY KEY ("job_id")
);
-- Create "node_diagnostics" table
CREATE TABLE "node_diagnostics" (
  "id" text NOT NULL,
  "job_id" text NOT NULL,
  "workflow_id" text NOT NULL,
  "node_id" text NULL,
  "ts" timestamptz NOT NULL,
  "schema" text NOT NULL,
  "code" text NOT NULL,
  "category" text NOT NULL,
  "severity" text NOT NULL,
  "message" bytea NOT NULL,
  "effective_disposition" text NULL,
  "action_type" text NULL,
  "feature_id" text NULL,
  "help" text NULL,
  "source_span_offset" bigint NULL,
  "source_span_length" bigint NULL,
  "aggregated_count" bigint NULL,
  "aggregated_sample_feature_ids" text[] NULL,
  PRIMARY KEY ("id")
);
-- Create index "node_diagnostics_job_id_node_id_ts_idx" to table: "node_diagnostics"
CREATE INDEX "node_diagnostics_job_id_node_id_ts_idx" ON "node_diagnostics" ("job_id", "node_id", "ts");
-- Create index "node_diagnostics_job_id_ts_idx" to table: "node_diagnostics"
CREATE INDEX "node_diagnostics_job_id_ts_idx" ON "node_diagnostics" ("job_id", "ts");
-- Set comment to column: "id" on table: "node_diagnostics"
COMMENT ON COLUMN "node_diagnostics"."id" IS 'deterministic, so Pub/Sub redeliveries and repeated terminal merges upsert';
-- Set comment to column: "node_id" on table: "node_diagnostics"
COMMENT ON COLUMN "node_diagnostics"."node_id" IS 'NULL for job-level diagnostics';
-- Set comment to column: "schema" on table: "node_diagnostics"
COMMENT ON COLUMN "node_diagnostics"."schema" IS 'job-complete.v1 marks a terminal row';
-- Set comment to column: "message" on table: "node_diagnostics"
COMMENT ON COLUMN "node_diagnostics"."message" IS 'bytea so a U+0000 from feature data round-trips';
