-- Diagnostics are returned oldest first, matching the Mongo repo's timestamp sort.

-- name: ListNodeDiagnosticsByJobNodeID :many
SELECT * FROM node_diagnostics
WHERE job_id = @job_id AND node_id = @node_id
ORDER BY ts, id;

-- name: ListJobLevelNodeDiagnostics :many
SELECT * FROM node_diagnostics
WHERE job_id = @job_id AND node_id IS NULL
ORDER BY ts, id;

-- name: ListNodeDiagnosticsByJobID :many
SELECT * FROM node_diagnostics
WHERE job_id = @job_id
ORDER BY ts, id;

-- name: UpsertNodeDiagnostic :exec
INSERT INTO node_diagnostics (
  id, job_id, workflow_id, node_id, ts, schema, code, category, severity, message,
  effective_disposition, action_type, feature_id, help,
  source_span_offset, source_span_length, aggregated_count, aggregated_sample_feature_ids
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
)
ON CONFLICT (id) DO UPDATE SET
  job_id                        = EXCLUDED.job_id,
  workflow_id                   = EXCLUDED.workflow_id,
  node_id                       = EXCLUDED.node_id,
  ts                            = EXCLUDED.ts,
  schema                        = EXCLUDED.schema,
  code                          = EXCLUDED.code,
  category                      = EXCLUDED.category,
  severity                      = EXCLUDED.severity,
  message                       = EXCLUDED.message,
  effective_disposition         = EXCLUDED.effective_disposition,
  action_type                   = EXCLUDED.action_type,
  feature_id                    = EXCLUDED.feature_id,
  help                          = EXCLUDED.help,
  source_span_offset            = EXCLUDED.source_span_offset,
  source_span_length            = EXCLUDED.source_span_length,
  aggregated_count              = EXCLUDED.aggregated_count,
  aggregated_sample_feature_ids = EXCLUDED.aggregated_sample_feature_ids;

-- name: GetJobDiagnosticSummary :one
SELECT * FROM job_diagnostic_summaries WHERE job_id = @job_id;

-- name: UpsertJobDiagnosticSummary :exec
INSERT INTO job_diagnostic_summaries (job_id, ts, dropped_event_count)
VALUES (@job_id, @ts, @dropped_event_count)
ON CONFLICT (job_id) DO UPDATE SET
  ts                  = EXCLUDED.ts,
  dropped_event_count = EXCLUDED.dropped_event_count;
