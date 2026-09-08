-- Create "config" table
CREATE TABLE "config" (
  "id" integer NOT NULL DEFAULT 1,
  "migration" bigint NOT NULL DEFAULT 0,
  "auth_cert" text NULL,
  "auth_key" text NULL,
  PRIMARY KEY ("id"),
  CONSTRAINT "config_singleton" CHECK (id = 1)
);
