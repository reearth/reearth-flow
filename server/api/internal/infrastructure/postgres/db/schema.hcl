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
