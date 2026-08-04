-- name: UpsertNodeExecution :exec
-- The execution ID is jobID:nodeID, so the primary key carries the per-node
-- uniqueness the Mongo path enforced with a jobId+nodeId filter.
INSERT INTO node_executions (
  id, job_id, node_id, status, started_at, completed_at
) VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO UPDATE SET
  job_id       = EXCLUDED.job_id,
  node_id      = EXCLUDED.node_id,
  status       = EXCLUDED.status,
  started_at   = EXCLUDED.started_at,
  completed_at = EXCLUDED.completed_at;
