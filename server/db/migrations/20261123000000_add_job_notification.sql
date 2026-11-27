-- Modify "jobs" table
ALTER TABLE "jobs" ADD COLUMN "notification_url" text NULL, ADD COLUMN "notified_at" timestamptz NULL, ADD COLUMN "artifacts_pending" boolean NOT NULL DEFAULT false, ADD COLUMN "notify_claimed_until" timestamptz NULL;
