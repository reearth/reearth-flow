-- name: GetProject :one
SELECT * FROM projects WHERE id = $1;

-- name: ListProjectsByIDs :many
SELECT * FROM projects WHERE id = ANY($1::text[]);

-- name: GetProjectByAlias :one
SELECT * FROM projects WHERE workspace_id = $1 AND alias = $2;

-- name: UpsertProject :exec
-- starred_by is written only when the row is created: afterwards stars change
-- through the star queries alone, so saving an edit made from a stale read
-- cannot drop a star added in the meantime.
INSERT INTO projects (
  id, workspace_id, workflow_id, name, description,
  is_archived, is_basic_auth_active, basic_auth_username, basic_auth_password,
  shared_token, updated_at, is_locked,
  alias, visibility, readme, license, topics, starred_by
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
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
  is_locked            = EXCLUDED.is_locked,
  alias                = EXCLUDED.alias,
  visibility           = EXCLUDED.visibility,
  readme               = EXCLUDED.readme,
  license              = EXCLUDED.license,
  topics               = EXCLUDED.topics;

-- name: DeleteProject :exec
DELETE FROM projects WHERE id = $1;

-- name: CountProjectsByWorkspace :one
SELECT count(*) FROM projects WHERE workspace_id = $1;

-- name: CountPublicProjectsByWorkspace :one
SELECT count(*) FROM projects WHERE workspace_id = $1 AND visibility = 'public';
