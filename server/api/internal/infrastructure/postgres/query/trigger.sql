-- name: GetTrigger :one
SELECT * FROM triggers WHERE id = $1;

-- name: ListTriggersByIDs :many
SELECT * FROM triggers WHERE id = ANY($1::text[]);

-- name: ListTriggersByDeployment :many
SELECT * FROM triggers WHERE deployment_id = $1;

-- name: UpsertTrigger :exec
INSERT INTO triggers (
  id, workspace_id, deployment_id, description, event_source,
  time_interval, auth_token_hash, auth_token_hint, enabled, last_triggered,
  variables, created_at, updated_at,
  last_refused_at, last_refusal_code, last_refusal_retry_until
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
ON CONFLICT (id) DO UPDATE SET
  workspace_id   = EXCLUDED.workspace_id,
  deployment_id  = EXCLUDED.deployment_id,
  description    = EXCLUDED.description,
  event_source   = EXCLUDED.event_source,
  time_interval  = EXCLUDED.time_interval,
  auth_token_hash = EXCLUDED.auth_token_hash,
  auth_token_hint = EXCLUDED.auth_token_hint,
  enabled        = EXCLUDED.enabled,
  last_triggered = EXCLUDED.last_triggered,
  variables      = EXCLUDED.variables,
  created_at     = EXCLUDED.created_at,
  updated_at     = EXCLUDED.updated_at,
  last_refused_at          = EXCLUDED.last_refused_at,
  last_refusal_code        = EXCLUDED.last_refusal_code,
  last_refusal_retry_until = EXCLUDED.last_refusal_retry_until;

-- name: UpdateTriggerLastRefusal :exec
UPDATE triggers SET
  last_refused_at          = $2,
  last_refusal_code        = $3,
  last_refusal_retry_until = $4
WHERE id = $1;

-- name: UpdateTriggerLastRefusalInWorkspaces :exec
UPDATE triggers SET
  last_refused_at          = @last_refused_at,
  last_refusal_code        = @last_refusal_code,
  last_refusal_retry_until = @last_refusal_retry_until
WHERE id = @id AND workspace_id = ANY(@workspace_ids::text[]);

-- name: DeleteTrigger :exec
DELETE FROM triggers WHERE id = $1;
