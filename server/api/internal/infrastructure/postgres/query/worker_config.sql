-- name: GetWorkerConfig :one
SELECT * FROM worker_configs WHERE id = $1;

-- name: ListWorkerConfigsByIDs :many
SELECT * FROM worker_configs WHERE id = ANY($1::text[]);

-- name: ListWorkerConfigs :many
SELECT * FROM worker_configs ORDER BY created_at ASC;

-- name: UpsertWorkerConfig :exec
INSERT INTO worker_configs (
  id, machine_type, compute_cpu_milli, compute_memory_mib, boot_disk_size_gb,
  task_count, max_concurrency, thread_pool_size, channel_buffer_size,
  feature_flush_threshold, node_status_delay_milli, created_at, updated_at
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
ON CONFLICT (id) DO UPDATE SET
  machine_type            = EXCLUDED.machine_type,
  compute_cpu_milli       = EXCLUDED.compute_cpu_milli,
  compute_memory_mib      = EXCLUDED.compute_memory_mib,
  boot_disk_size_gb       = EXCLUDED.boot_disk_size_gb,
  task_count              = EXCLUDED.task_count,
  max_concurrency         = EXCLUDED.max_concurrency,
  thread_pool_size        = EXCLUDED.thread_pool_size,
  channel_buffer_size     = EXCLUDED.channel_buffer_size,
  feature_flush_threshold = EXCLUDED.feature_flush_threshold,
  node_status_delay_milli = EXCLUDED.node_status_delay_milli,
  created_at              = EXCLUDED.created_at,
  updated_at              = EXCLUDED.updated_at;

-- name: DeleteWorkerConfig :exec
DELETE FROM worker_configs WHERE id = $1;
