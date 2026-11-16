-- Modify "jobs" table
ALTER TABLE "jobs" ADD COLUMN "trigger_id" text NULL, ADD COLUMN "scheduled_at" timestamptz NULL;
-- Create index "jobs_trigger_id_scheduled_at_idx" to table: "jobs"
CREATE UNIQUE INDEX "jobs_trigger_id_scheduled_at_idx" ON "jobs" ("trigger_id", "scheduled_at");
