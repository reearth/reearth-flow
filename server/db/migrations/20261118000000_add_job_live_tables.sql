-- Create "job_complete_events" table
CREATE UNLOGGED TABLE "job_complete_events" (
  "job_id" text NOT NULL,
  "data" json NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("job_id")
);
-- Create index "job_complete_events_created_at_idx" to table: "job_complete_events"
CREATE INDEX "job_complete_events_created_at_idx" ON "job_complete_events" ("created_at");
-- Create "job_diagnostics" table
CREATE UNLOGGED TABLE "job_diagnostics" (
  "id" bigserial NOT NULL,
  "job_id" text NOT NULL,
  "node_id" text NULL,
  "data" json NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Create index "job_diagnostics_created_at_idx" to table: "job_diagnostics"
CREATE INDEX "job_diagnostics_created_at_idx" ON "job_diagnostics" ("created_at");
-- Create index "job_diagnostics_job_id_node_id_id_idx" to table: "job_diagnostics"
CREATE INDEX "job_diagnostics_job_id_node_id_id_idx" ON "job_diagnostics" ("job_id", "node_id", "id");
-- Set comment to column: "id" on table: "job_diagnostics"
COMMENT ON COLUMN "job_diagnostics"."id" IS 'insertion order; readers return newest first, matching the Redis LPUSH lists';
-- Set comment to column: "node_id" on table: "job_diagnostics"
COMMENT ON COLUMN "job_diagnostics"."node_id" IS 'NULL for job-level diagnostics (the Redis _job segment)';
-- Create "job_logs" table
CREATE UNLOGGED TABLE "job_logs" (
  "id" bigserial NOT NULL,
  "job_id" text NOT NULL,
  "workflow_id" text NOT NULL,
  "node_id" text NULL,
  "ts" timestamptz NOT NULL,
  "level" text NOT NULL,
  "message" bytea NOT NULL,
  "payload_hash" bytea NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Create index "job_logs_created_at_idx" to table: "job_logs"
CREATE INDEX "job_logs_created_at_idx" ON "job_logs" ("created_at");
-- Create index "job_logs_job_id_payload_hash_key" to table: "job_logs"
CREATE UNIQUE INDEX "job_logs_job_id_payload_hash_key" ON "job_logs" ("job_id", "payload_hash");
-- Create index "job_logs_job_id_ts_idx" to table: "job_logs"
CREATE INDEX "job_logs_job_id_ts_idx" ON "job_logs" ("job_id", "ts");
-- Set comment to column: "payload_hash" on table: "job_logs"
COMMENT ON COLUMN "job_logs"."payload_hash" IS 'sha256 of the serialized event; dedupes Pub/Sub redelivery';
-- Create "job_user_facing_logs" table
CREATE UNLOGGED TABLE "job_user_facing_logs" (
  "id" bigserial NOT NULL,
  "job_id" text NOT NULL,
  "workflow_id" text NOT NULL,
  "node_id" text NULL,
  "node_name" text NULL,
  "ts" timestamptz NOT NULL,
  "level" text NOT NULL,
  "message" bytea NOT NULL,
  "metadata" jsonb NULL,
  "payload_hash" bytea NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Create index "job_user_facing_logs_created_at_idx" to table: "job_user_facing_logs"
CREATE INDEX "job_user_facing_logs_created_at_idx" ON "job_user_facing_logs" ("created_at");
-- Create index "job_user_facing_logs_job_id_payload_hash_key" to table: "job_user_facing_logs"
CREATE UNIQUE INDEX "job_user_facing_logs_job_id_payload_hash_key" ON "job_user_facing_logs" ("job_id", "payload_hash");
-- Create index "job_user_facing_logs_job_id_ts_idx" to table: "job_user_facing_logs"
CREATE INDEX "job_user_facing_logs_job_id_ts_idx" ON "job_user_facing_logs" ("job_id", "ts");
-- Set comment to column: "payload_hash" on table: "job_user_facing_logs"
COMMENT ON COLUMN "job_user_facing_logs"."payload_hash" IS 'sha256 of the serialized event; dedupes Pub/Sub redelivery';
