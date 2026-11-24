-- Modify "jobs" table
ALTER TABLE "jobs" ADD COLUMN "notification_url" text NULL, ADD COLUMN "notified_at" timestamptz NULL;
