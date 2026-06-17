-- name: GetJob :one
SELECT * FROM jobs WHERE id = $1;

-- name: ListJobsByIDs :many
SELECT * FROM jobs WHERE id = ANY($1::text[]);

-- name: UpsertJob :exec
INSERT INTO jobs (
  id, workspace_id, deployment_id, project_id, project_version,
  gcp_job_id, logs_url, worker_logs_url, user_facing_logs_url,
  status, batch_status, worker_status,
  started_at, completed_at, metadata_url, output_urls, debug, parameters
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
ON CONFLICT (id) DO UPDATE SET
  workspace_id         = EXCLUDED.workspace_id,
  deployment_id        = EXCLUDED.deployment_id,
  project_id           = EXCLUDED.project_id,
  project_version      = EXCLUDED.project_version,
  gcp_job_id           = EXCLUDED.gcp_job_id,
  logs_url             = EXCLUDED.logs_url,
  worker_logs_url      = EXCLUDED.worker_logs_url,
  user_facing_logs_url = EXCLUDED.user_facing_logs_url,
  status               = EXCLUDED.status,
  batch_status         = EXCLUDED.batch_status,
  worker_status        = EXCLUDED.worker_status,
  started_at           = EXCLUDED.started_at,
  completed_at         = EXCLUDED.completed_at,
  metadata_url         = EXCLUDED.metadata_url,
  output_urls          = EXCLUDED.output_urls,
  debug                = EXCLUDED.debug,
  parameters           = EXCLUDED.parameters;

-- name: DeleteJob :exec
DELETE FROM jobs WHERE id = $1;

-- name: DeleteJobsByProject :exec
DELETE FROM jobs WHERE project_id = $1 AND debug = true;
