-- name: GetProject :one
SELECT * FROM projects WHERE id = $1;

-- name: ListProjectsByIDs :many
SELECT * FROM projects WHERE id = ANY($1::text[]);

-- name: UpsertProject :exec
INSERT INTO projects (
  id, workspace_id, workflow_id, name, description,
  is_archived, is_basic_auth_active, basic_auth_username, basic_auth_password,
  shared_token, updated_at, is_locked
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
ON CONFLICT (id) DO UPDATE SET
  workspace_id         = EXCLUDED.workspace_id,
  workflow_id          = EXCLUDED.workflow_id,
  name                 = EXCLUDED.name,
  description          = EXCLUDED.description,
  is_archived          = EXCLUDED.is_archived,
  is_basic_auth_active = EXCLUDED.is_basic_auth_active,
  basic_auth_username  = EXCLUDED.basic_auth_username,
  basic_auth_password  = EXCLUDED.basic_auth_password,
  shared_token         = EXCLUDED.shared_token,
  updated_at           = EXCLUDED.updated_at,
  is_locked            = EXCLUDED.is_locked;

-- name: DeleteProject :exec
DELETE FROM projects WHERE id = $1;

-- name: CountProjectsByWorkspace :one
SELECT count(*) FROM projects WHERE workspace_id = $1;

-- name: CountPublicProjectsByWorkspace :one
-- Flow's project domain has no publishment-status concept. The Mongo impl
-- counted a raw `publishmentstatus` field that flow never writes (it is not in
-- mongodoc.ProjectDocument), so it effectively returns 0 for flow-owned data.
-- Mirror that faithfully rather than inventing an unowned column.
SELECT count(*) FROM projects WHERE workspace_id = $1 AND false;
