-- name: GetDeployment :one
SELECT * FROM deployments WHERE id = $1;

-- name: ListDeploymentsByIDs :many
SELECT * FROM deployments WHERE id = ANY($1::text[]);

-- name: UpsertDeployment :exec
INSERT INTO deployments (
  id, workspace_id, project_id, workflow_url, description,
  version, updated_at, head_id, is_head
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
ON CONFLICT (id) DO UPDATE SET
  workspace_id = EXCLUDED.workspace_id,
  project_id   = EXCLUDED.project_id,
  workflow_url = EXCLUDED.workflow_url,
  description  = EXCLUDED.description,
  version      = EXCLUDED.version,
  updated_at   = EXCLUDED.updated_at,
  head_id      = EXCLUDED.head_id,
  is_head      = EXCLUDED.is_head;

-- name: DeleteDeployment :exec
DELETE FROM deployments WHERE id = $1;
