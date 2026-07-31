-- Create "auth_requests" table
CREATE TABLE "auth_requests" (
  "id" text NOT NULL,
  "client_id" text NOT NULL DEFAULT '',
  "subject" text NOT NULL DEFAULT '',
  "code" text NOT NULL DEFAULT '',
  "state" text NOT NULL DEFAULT '',
  "response_type" text NOT NULL DEFAULT '',
  "scopes" jsonb NULL,
  "audiences" jsonb NULL,
  "redirect_uri" text NOT NULL DEFAULT '',
  "nonce" text NOT NULL DEFAULT '',
  "code_challenge" jsonb NULL,
  "authorized_at" timestamptz NULL,
  PRIMARY KEY ("id")
);
-- Create index "auth_requests_code_idx" to table: "auth_requests"
CREATE INDEX "auth_requests_code_idx" ON "auth_requests" ("code");
-- Create index "auth_requests_subject_idx" to table: "auth_requests"
CREATE INDEX "auth_requests_subject_idx" ON "auth_requests" ("subject");
