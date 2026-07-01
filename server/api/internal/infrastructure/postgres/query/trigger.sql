-- name: GetTrigger :one
SELECT * FROM triggers WHERE id = $1;

-- name: ListTriggersByIDs :many
SELECT * FROM triggers WHERE id = ANY($1::text[]);

-- name: ListTriggersByDeployment :many
SELECT * FROM triggers WHERE deployment_id = $1;

-- name: UpsertTrigger :exec
INSERT INTO triggers (
  id, workspace_id, deployment_id, description, event_source,
  time_interval, auth_token, enabled, last_triggered, variables,
  created_at, updated_at
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
ON CONFLICT (id) DO UPDATE SET
  workspace_id   = EXCLUDED.workspace_id,
  deployment_id  = EXCLUDED.deployment_id,
  description    = EXCLUDED.description,
  event_source   = EXCLUDED.event_source,
  time_interval  = EXCLUDED.time_interval,
  auth_token     = EXCLUDED.auth_token,
  enabled        = EXCLUDED.enabled,
  last_triggered = EXCLUDED.last_triggered,
  variables      = EXCLUDED.variables,
  created_at     = EXCLUDED.created_at,
  updated_at     = EXCLUDED.updated_at;

-- name: DeleteTrigger :exec
DELETE FROM triggers WHERE id = $1;
