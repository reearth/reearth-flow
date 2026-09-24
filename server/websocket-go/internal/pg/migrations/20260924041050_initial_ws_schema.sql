-- Create "ws_instance" table
CREATE UNLOGGED TABLE "ws_instance" (
  "doc_id" text NOT NULL,
  "client_id" bigint NOT NULL,
  "seen_at" timestamptz NOT NULL,
  PRIMARY KEY ("doc_id", "client_id")
);
-- Create index "ws_instance_seen_at_idx" to table: "ws_instance"
CREATE INDEX "ws_instance_seen_at_idx" ON "ws_instance" ("seen_at");
-- Create "ws_lock" table
CREATE UNLOGGED TABLE "ws_lock" (
  "key" text NOT NULL,
  "owner" text NOT NULL,
  "expires_at" timestamptz NOT NULL,
  PRIMARY KEY ("key")
);
-- Create index "ws_lock_expires_at_idx" to table: "ws_lock"
CREATE INDEX "ws_lock_expires_at_idx" ON "ws_lock" ("expires_at");
-- Create "ws_stream" table
CREATE UNLOGGED TABLE "ws_stream" (
  "id" bigserial NOT NULL,
  "doc_id" text NOT NULL,
  "kind" smallint NOT NULL,
  "data" bytea NOT NULL,
  "client_id" bigint NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY ("id")
);
-- Create index "ws_stream_created_at_idx" to table: "ws_stream"
CREATE INDEX "ws_stream_created_at_idx" ON "ws_stream" ("created_at");
-- Create index "ws_stream_doc_id_idx" to table: "ws_stream"
CREATE INDEX "ws_stream_doc_id_idx" ON "ws_stream" ("doc_id", "id");
-- Set comment to column: "kind" on table: "ws_stream"
COMMENT ON COLUMN "ws_stream"."kind" IS '0 = sync (document update), 1 = awareness (presence)';
-- Set comment to column: "client_id" on table: "ws_stream"
COMMENT ON COLUMN "ws_stream"."client_id" IS 'writing instance, so a reader can skip its own rows';
