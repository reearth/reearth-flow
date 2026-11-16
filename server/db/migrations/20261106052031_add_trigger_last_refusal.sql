-- Modify "triggers" table
ALTER TABLE "triggers" ADD COLUMN "last_refused_at" timestamptz NULL, ADD COLUMN "last_refusal_code" text NULL, ADD COLUMN "last_refusal_retry_until" timestamptz NULL;
