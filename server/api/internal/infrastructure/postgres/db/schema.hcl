schema "public" {}

table "config" {
  schema = schema.public
  column "id" {
    type    = integer
    default = 1
  }
  column "migration" {
    type    = bigint
    default = 0
  }
  column "auth_cert" {
    type = text
    null = true
  }
  column "auth_key" {
    type = text
    null = true
  }
  primary_key {
    columns = [column.id]
  }
  check "config_singleton" {
    expr = "id = 1"
  }
}

table "triggers" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "workspace_id" {
    type = text
  }
  column "deployment_id" {
    type = text
  }
  column "description" {
    type    = text
    default = ""
  }
  column "event_source" {
    type = text
  }
  column "time_interval" {
    type = text
    null = true
  }
  column "auth_token" {
    type = text
    null = true
  }
  column "enabled" {
    type    = boolean
    default = false
  }
  column "last_triggered" {
    type = timestamptz
    null = true
  }
  column "variables" {
    type = jsonb
    null = true
  }
  column "created_at" {
    type = timestamptz
  }
  column "updated_at" {
    type = timestamptz
  }

  primary_key {
    columns = [column.id]
  }

  index "triggers_workspace_id_idx" {
    columns = [column.workspace_id]
  }
  index "triggers_deployment_id_idx" {
    columns = [column.deployment_id]
  }
}

table "parameters" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "project_id" {
    type = text
  }
  column "name" {
    type = text
  }
  column "type" {
    type = text
  }
  column "index" {
    type    = integer
    default = 0
  }
  column "required" {
    type    = boolean
    default = false
  }
  column "public" {
    type    = boolean
    default = false
  }
  column "default_value" {
    type = jsonb
    null = true
  }
  column "config" {
    type = jsonb
    null = true
  }
  column "created_at" {
    type = timestamptz
  }
  column "updated_at" {
    type = timestamptz
  }

  primary_key {
    columns = [column.id]
  }

  index "parameters_project_id_idx" {
    columns = [column.project_id]
  }
}

table "project_accesses" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "project_id" {
    type = text
  }
  column "token" {
    type    = text
    default = ""
  }
  column "is_public" {
    type    = boolean
    default = false
  }

  primary_key {
    columns = [column.id]
  }

  index "project_accesses_project_id_idx" {
    columns = [column.project_id]
    unique  = true
  }
  index "project_accesses_token_idx" {
    columns = [column.token]
  }
}

table "workflows" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "project_id" {
    type = text
  }
  column "workspace_id" {
    type = text
  }
  column "url" {
    type    = text
    default = ""
  }

  primary_key {
    columns = [column.id]
  }

  index "workflows_workspace_id_idx" {
    columns = [column.workspace_id]
  }
  index "workflows_project_id_idx" {
    columns = [column.project_id]
  }
}

table "edge_executions" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "edge_id" {
    type = text
  }
  column "job_id" {
    type = text
  }
  column "intermediate_data_url" {
    type = text
    null = true
  }

  primary_key {
    columns = [column.id]
  }

  index "edge_executions_job_id_idx" {
    columns = [column.job_id]
  }
  index "edge_executions_job_id_edge_id_idx" {
    columns = [column.job_id, column.edge_id]
  }
}

table "node_executions" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "job_id" {
    type = text
  }
  column "node_id" {
    type = text
  }
  column "status" {
    type    = text
    default = "PENDING"
  }
  column "started_at" {
    type = timestamptz
    null = true
  }
  column "completed_at" {
    type = timestamptz
    null = true
  }

  primary_key {
    columns = [column.id]
  }

  index "node_executions_job_id_idx" {
    columns = [column.job_id]
  }
  index "node_executions_job_id_node_id_idx" {
    columns = [column.job_id, column.node_id]
  }
}

table "worker_configs" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "machine_type" {
    type = text
    null = true
  }
  column "compute_cpu_milli" {
    type = integer
    null = true
  }
  column "compute_memory_mib" {
    type = integer
    null = true
  }
  column "boot_disk_size_gb" {
    type = integer
    null = true
  }
  column "task_count" {
    type = integer
    null = true
  }
  column "max_concurrency" {
    type = integer
    null = true
  }
  column "thread_pool_size" {
    type = integer
    null = true
  }
  column "channel_buffer_size" {
    type = integer
    null = true
  }
  column "feature_flush_threshold" {
    type = integer
    null = true
  }
  column "node_status_delay_milli" {
    type = integer
    null = true
  }
  column "created_at" {
    type = timestamptz
  }
  column "updated_at" {
    type = timestamptz
  }

  primary_key {
    columns = [column.id]
  }
}

table "deployments" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "workspace_id" {
    type = text
  }
  column "project_id" {
    type = text
    null = true
  }
  column "workflow_url" {
    type    = text
    default = ""
  }
  column "description" {
    type    = text
    default = ""
  }
  column "version" {
    type    = text
    default = ""
  }
  column "updated_at" {
    type = timestamptz
  }
  column "head_id" {
    type = text
    null = true
  }
  column "is_head" {
    type    = boolean
    default = false
  }

  primary_key {
    columns = [column.id]
  }

  index "deployments_workspace_id_idx" {
    columns = [column.workspace_id]
  }
  index "deployments_project_id_idx" {
    columns = [column.project_id]
  }
  index "deployments_workspace_id_is_head_idx" {
    columns = [column.workspace_id, column.is_head]
  }
}

table "jobs" {
  schema = schema.public

  column "id" {
    type = text
  }
  column "workspace_id" {
    type = text
  }
  column "deployment_id" {
    type = text
    null = true
  }
  column "project_id" {
    type = text
    null = true
  }
  column "project_version" {
    type = integer
    null = true
  }
  column "gcp_job_id" {
    type    = text
    default = ""
  }
  column "logs_url" {
    type    = text
    default = ""
  }
  column "worker_logs_url" {
    type    = text
    default = ""
  }
  column "user_facing_logs_url" {
    type    = text
    default = ""
  }
  column "status" {
    type    = text
    default = ""
  }
  column "batch_status" {
    type = text
    null = true
  }
  column "worker_status" {
    type = text
    null = true
  }
  column "started_at" {
    type = timestamptz
  }
  column "completed_at" {
    type = timestamptz
    null = true
  }
  column "metadata_url" {
    type    = text
    default = ""
  }
  column "output_urls" {
    type = jsonb
    null = true
  }
  column "debug" {
    type = boolean
    null = true
  }
  column "parameters" {
    type = jsonb
    null = true
  }

  primary_key {
    columns = [column.id]
  }

  index "jobs_workspace_id_idx" {
    columns = [column.workspace_id]
  }
  index "jobs_deployment_id_idx" {
    columns = [column.deployment_id]
  }
  index "jobs_project_id_idx" {
    columns = [column.project_id]
  }
  index "jobs_workspace_id_debug_idx" {
    columns = [column.workspace_id, column.debug]
  }
}
