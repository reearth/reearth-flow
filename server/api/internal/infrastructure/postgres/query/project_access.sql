-- name: GetProjectAccessByProjectID :one
SELECT * FROM project_accesses WHERE project_id = $1;

-- name: GetProjectAccessByToken :one
SELECT * FROM project_accesses WHERE token = $1;

-- name: UpsertProjectAccess :exec
INSERT INTO project_accesses (id, project_id, token, is_public)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET
  project_id = EXCLUDED.project_id,
  token      = EXCLUDED.token,
  is_public  = EXCLUDED.is_public;
