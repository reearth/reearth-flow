-- name: GetWorkflow :one
SELECT * FROM workflows WHERE id = $1;

-- name: UpsertWorkflow :exec
INSERT INTO workflows (id, project_id, workspace_id, url)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET
  project_id   = EXCLUDED.project_id,
  workspace_id = EXCLUDED.workspace_id,
  url          = EXCLUDED.url;

-- name: DeleteWorkflow :exec
DELETE FROM workflows WHERE id = $1;
