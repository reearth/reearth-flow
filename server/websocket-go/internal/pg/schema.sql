-- Coordination schema for the websocket-go Postgres backend.
--
-- Every table is UNLOGGED: writes skip the WAL, so these tables are emptied by an
-- unclean shutdown and are absent from replicas. That is acceptable because the
-- durable document lives in GCS and the relay data it replaces already expired
-- after 6 hours in Redis. It is NOT acceptable for anything else, so nothing else
-- belongs in this schema.
--
-- Applied idempotently at startup by migrations.go. Statements must therefore stay
-- IF NOT EXISTS / re-runnable.

CREATE UNLOGGED TABLE IF NOT EXISTS ws_stream (
  id         bigserial   PRIMARY KEY,
  doc_id     text        NOT NULL,
  kind       smallint    NOT NULL,
  data       bytea       NOT NULL,
  client_id  bigint      NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

-- The reader's hot path: one document, everything after a cursor, in id order.
CREATE INDEX IF NOT EXISTS ws_stream_doc_id_idx ON ws_stream (doc_id, id);
-- The janitor's retention sweep.
CREATE INDEX IF NOT EXISTS ws_stream_created_at_idx ON ws_stream (created_at);

CREATE UNLOGGED TABLE IF NOT EXISTS ws_instance (
  doc_id    text        NOT NULL,
  client_id bigint      NOT NULL,
  seen_at   timestamptz NOT NULL,
  PRIMARY KEY (doc_id, client_id)
);

CREATE INDEX IF NOT EXISTS ws_instance_seen_at_idx ON ws_instance (seen_at);

-- ws_lock replaces Redis SET NX PX for locks whose critical section is NOT a
-- Postgres transaction.
--
-- The read lock and the OID lock both wrap arbitrary work — the read lock spans a
-- GCS flush — so pg_advisory_xact_lock cannot express them: it would mean holding
-- a transaction (and its connection) open across network I/O, which
-- idle_in_transaction_session_timeout is specifically configured to kill. A row
-- with an explicit expiry reproduces the Redis semantics the callers were written
-- against, including the TTL they rely on to recover from a dead holder.
--
-- The election in queries.go is the one lock that IS a short transaction, and that
-- one does use pg_advisory_xact_lock.
CREATE UNLOGGED TABLE IF NOT EXISTS ws_lock (
  key        text        PRIMARY KEY,
  owner      text        NOT NULL,
  expires_at timestamptz NOT NULL
);

CREATE INDEX IF NOT EXISTS ws_lock_expires_at_idx ON ws_lock (expires_at);
