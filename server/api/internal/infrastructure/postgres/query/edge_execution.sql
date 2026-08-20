-- name: GetEdgeExecution :one
SELECT * FROM edge_executions WHERE id = $1;

-- name: GetEdgeExecutionByJobEdgeID :one
SELECT * FROM edge_executions WHERE job_id = $1 AND edge_id = $2;

-- name: ListEdgeExecutionsByJobID :many
SELECT * FROM edge_executions WHERE job_id = $1;

-- name: UpsertEdgeExecution :exec
INSERT INTO edge_executions (
  id, edge_id, job_id, intermediate_data_url
) VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET
  edge_id               = EXCLUDED.edge_id,
  job_id                = EXCLUDED.job_id,
  intermediate_data_url = EXCLUDED.intermediate_data_url;
