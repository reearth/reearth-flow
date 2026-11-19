-- Transient job live data (REEARTH_FLOW_LIVE_BACKEND=postgres). Written by the
-- subscriber, read here; see the unlogged section of db/schema.hcl.

-- name: ListJobLogs :many
SELECT job_id, node_id, ts, level, message
FROM job_logs
WHERE job_id = @job_id AND ts >= @since AND ts <= @until
ORDER BY ts, id;

-- name: ListJobLogsByJobIDs :many
SELECT job_id, node_id, ts, level, message
FROM job_logs
WHERE job_id = ANY(@job_ids::text[]) AND ts >= @since AND ts <= @until
ORDER BY job_id, ts, id;

-- name: ListJobUserFacingLogs :many
SELECT job_id, node_id, node_name, ts, level, message, metadata
FROM job_user_facing_logs
WHERE job_id = @job_id AND ts >= @since AND ts <= @until
ORDER BY ts, id;

-- Diagnostics are returned newest first, matching the LPUSH'd Redis lists.

-- name: ListNodeDiagnostics :many
SELECT data FROM job_diagnostics
WHERE job_id = @job_id AND node_id = @node_id
ORDER BY id DESC;

-- name: ListJobLevelDiagnostics :many
SELECT data FROM job_diagnostics
WHERE job_id = @job_id AND node_id IS NULL
ORDER BY id DESC;

-- name: ListJobDiagnostics :many
SELECT data FROM job_diagnostics
WHERE job_id = @job_id
ORDER BY id DESC;

-- name: GetJobCompleteEvent :one
SELECT data FROM job_complete_events WHERE job_id = @job_id;

-- name: DeleteJobCompleteEvent :exec
DELETE FROM job_complete_events WHERE job_id = @job_id;
