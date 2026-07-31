-- Create "worker_configs" table
CREATE TABLE "worker_configs" (
  "id" text NOT NULL,
  "machine_type" text NULL,
  "compute_cpu_milli" integer NULL,
  "compute_memory_mib" integer NULL,
  "boot_disk_size_gb" integer NULL,
  "task_count" integer NULL,
  "max_concurrency" integer NULL,
  "thread_pool_size" integer NULL,
  "channel_buffer_size" integer NULL,
  "feature_flush_threshold" integer NULL,
  "node_status_delay_milli" integer NULL,
  "created_at" timestamptz NOT NULL,
  "updated_at" timestamptz NOT NULL,
  PRIMARY KEY ("id")
);
