// Coordination schema for server/websocket-go: the relay's cross-instance state.
//
// Separate from schema.hcl because it has a different lifecycle and a different
// database. Every table is UNLOGGED — writes skip the WAL, so a crash empties them
// and they never reach a replica. That is correct here: the durable document lives
// in GCS and this data expires within hours. Nothing system-of-record belongs here.

schema "public" {}

table "ws_stream" {
  schema   = schema.public
  unlogged = true

  column "id" {
    type = bigserial
  }
  column "doc_id" {
    type = text
  }
  column "kind" {
    type    = smallint
    comment = "0 = sync (document update), 1 = awareness (presence)"
  }
  column "data" {
    type = bytea
  }
  column "client_id" {
    type    = bigint
    comment = "writing instance, so a reader can skip its own rows"
  }
  column "created_at" {
    type    = timestamptz
    default = sql("now()")
  }

  primary_key {
    columns = [column.id]
  }

  // The reader's hot path: one document, everything after a cursor, in id order.
  index "ws_stream_doc_id_idx" {
    columns = [column.doc_id, column.id]
  }
  // The janitor's retention sweep.
  index "ws_stream_created_at_idx" {
    columns = [column.created_at]
  }
}

table "ws_instance" {
  schema   = schema.public
  unlogged = true

  column "doc_id" {
    type = text
  }
  column "client_id" {
    type = bigint
  }
  column "seen_at" {
    type = timestamptz
  }

  primary_key {
    columns = [column.doc_id, column.client_id]
  }

  index "ws_instance_seen_at_idx" {
    columns = [column.seen_at]
  }
}

// ws_lock replaces Redis SET NX PX for locks whose critical section is NOT a
// Postgres transaction — the read lock spans a GCS round trip, so an advisory lock
// would hold a transaction open across network I/O.
table "ws_lock" {
  schema   = schema.public
  unlogged = true

  column "key" {
    type = text
  }
  column "owner" {
    type = text
  }
  column "expires_at" {
    type = timestamptz
  }

  primary_key {
    columns = [column.key]
  }

  index "ws_lock_expires_at_idx" {
    columns = [column.expires_at]
  }
}
