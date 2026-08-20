-- Create "triggers" table
CREATE TABLE "triggers" (
  "id" text NOT NULL,
  "workspace_id" text NOT NULL,
  "deployment_id" text NOT NULL,
  "description" text NOT NULL DEFAULT '',
  "event_source" text NOT NULL,
  "time_interval" text NULL,
  "auth_token" text NULL,
  "enabled" boolean NOT NULL DEFAULT false,
  "last_triggered" timestamptz NULL,
  "variables" jsonb NULL,
  "created_at" timestamptz NOT NULL,
  "updated_at" timestamptz NOT NULL,
  PRIMARY KEY ("id")
);
-- Create index "triggers_deployment_id_idx" to table: "triggers"
CREATE INDEX "triggers_deployment_id_idx" ON "triggers" ("deployment_id");
-- Create index "triggers_workspace_id_idx" to table: "triggers"
CREATE INDEX "triggers_workspace_id_idx" ON "triggers" ("workspace_id");
