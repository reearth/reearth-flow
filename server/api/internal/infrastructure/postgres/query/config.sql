-- name: GetConfig :one
SELECT migration, auth_cert, auth_key FROM config WHERE id = 1;

-- name: UpsertConfig :exec
INSERT INTO config (id, migration, auth_cert, auth_key)
VALUES (1, $1, $2, $3)
ON CONFLICT (id) DO UPDATE SET
  migration = EXCLUDED.migration,
  auth_cert = EXCLUDED.auth_cert,
  auth_key  = EXCLUDED.auth_key;

-- name: UpsertConfigAuth :exec
INSERT INTO config (id, auth_cert, auth_key)
VALUES (1, $1, $2)
ON CONFLICT (id) DO UPDATE SET
  auth_cert = EXCLUDED.auth_cert,
  auth_key  = EXCLUDED.auth_key;
