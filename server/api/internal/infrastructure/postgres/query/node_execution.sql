-- name: GetNodeExecutionByJobNodeID :one
SELECT * FROM node_executions WHERE job_id = $1 AND node_id = $2;

-- name: UpsertNodeExecution :exec
INSERT INTO node_executions (
  id, job_id, node_id, status, started_at, completed_at
) VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO UPDATE SET
  job_id       = EXCLUDED.job_id,
  node_id      = EXCLUDED.node_id,
  status       = EXCLUDED.status,
  started_at   = EXCLUDED.started_at,
  completed_at = EXCLUDED.completed_at;
