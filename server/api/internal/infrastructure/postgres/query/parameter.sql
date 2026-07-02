-- name: GetParameter :one
SELECT * FROM parameters WHERE id = $1;

-- name: ListParametersByIDs :many
SELECT * FROM parameters WHERE id = ANY($1::text[]);

-- name: ListParametersByProject :many
SELECT * FROM parameters WHERE project_id = $1 ORDER BY index ASC;

-- name: UpsertParameter :exec
INSERT INTO parameters (
  id, project_id, name, type, index, required, public,
  default_value, config, created_at, updated_at
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
ON CONFLICT (id) DO UPDATE SET
  project_id    = EXCLUDED.project_id,
  name          = EXCLUDED.name,
  type          = EXCLUDED.type,
  index         = EXCLUDED.index,
  required      = EXCLUDED.required,
  public        = EXCLUDED.public,
  default_value = EXCLUDED.default_value,
  config        = EXCLUDED.config,
  created_at    = EXCLUDED.created_at,
  updated_at    = EXCLUDED.updated_at;

-- name: DeleteParameter :exec
DELETE FROM parameters WHERE id = $1;

-- name: DeleteParametersByIDs :exec
DELETE FROM parameters WHERE id = ANY($1::text[]);

-- name: DeleteParametersByProject :exec
DELETE FROM parameters WHERE project_id = $1;
