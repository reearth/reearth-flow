-- Modify "triggers" table
ALTER TABLE "triggers" ADD COLUMN "auth_token_hash" text NULL, ADD COLUMN "auth_token_hint" text NULL;
-- Hash existing tokens in place (see trigger.HashAuthToken) so callers keep using the token they have; they get no hint, which could give a short token away
UPDATE "triggers" SET "auth_token_hash" = encode(sha256(convert_to("auth_token", 'UTF8')), 'hex') WHERE "auth_token" <> '';
-- No plain-text token is kept
ALTER TABLE "triggers" DROP COLUMN "auth_token";
