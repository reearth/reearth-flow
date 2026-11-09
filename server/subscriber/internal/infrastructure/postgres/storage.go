// Package postgres writes the job live data (logs, user-facing logs,
// diagnostics, job-complete events) to the API database's unlogged job_* tables,
// for environments that run without Redis (REEARTH_FLOW_SUBSCRIBER_LIVE_BACKEND=postgres).
// The schema lives in server/db/schema.hcl; the API reads the same tables.
package postgres

import (
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"strings"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

// Querier is the subset of pgxpool.Pool this package uses, so tests can pass a
// single connection. Satisfied by *pgxpool.Pool, *pgx.Conn and pgx.Tx.
type Querier interface {
	Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error)
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row
}

type Storage struct {
	q Querier
}

func NewStorage(q Querier) *Storage {
	return &Storage{q: q}
}

// Messages are bytea and payloads json (not text/jsonb) so a U+0000 round-trips instead of failing the insert.

// Logs are deduplicated on (job_id, payload_hash): Pub/Sub delivers at least
// once, and a redelivered message serializes to the same bytes. Redis tolerated
// the duplicate by overwriting the same per-line key; here it is dropped.
const (
	insertLogSQL = `INSERT INTO job_logs (job_id, workflow_id, node_id, ts, level, message, payload_hash)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (job_id, payload_hash) DO NOTHING`

	insertUserFacingLogSQL = `INSERT INTO job_user_facing_logs (job_id, workflow_id, node_id, node_name, ts, level, message, payload_hash)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT (job_id, payload_hash) DO NOTHING`

	// Diagnostics are appended without dedupe, matching the Redis LPUSH lists.
	insertDiagnosticSQL = `INSERT INTO job_diagnostics (job_id, node_id, data) VALUES ($1, $2, $3)`

	// A repeated event replaces the stored one and restarts its retention, like
	// the Redis SET with a TTL it stands in for.
	upsertJobCompleteSQL = `INSERT INTO job_complete_events (job_id, data) VALUES ($1, $2)
ON CONFLICT (job_id) DO UPDATE SET data = EXCLUDED.data, created_at = now()`
)

func (s *Storage) SaveLog(ctx context.Context, event *domainLog.LogEvent) error {
	if event == nil {
		return errors.New("cannot save nil log event")
	}
	serialized, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}
	hash := sha256.Sum256(serialized)
	if _, err := s.q.Exec(ctx, insertLogSQL,
		event.JobID, event.WorkflowID, nullIfEmpty(event.NodeID), event.Timestamp.UTC(),
		string(event.LogLevel), []byte(event.Message), hash[:],
	); err != nil {
		return fmt.Errorf("failed to insert log: %w", err)
	}
	return nil
}

func (s *Storage) SaveUserFacingLog(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	if event == nil {
		return errors.New("cannot save nil user facing log event")
	}
	serialized, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal user facing log event: %w", err)
	}
	hash := sha256.Sum256(serialized)
	if _, err := s.q.Exec(ctx, insertUserFacingLogSQL,
		event.JobID, event.WorkflowID, nullIfEmpty(event.NodeID), event.NodeName, event.Timestamp.UTC(),
		string(event.Level), []byte(event.Message), hash[:],
	); err != nil {
		return fmt.Errorf("failed to insert user facing log: %w", err)
	}
	return nil
}

func (s *Storage) SaveDiagnostic(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	if event == nil {
		return errors.New("cannot save nil diagnostic event")
	}
	serialized, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal diagnostic event: %w", err)
	}
	// A nil or empty node ID is a job-level diagnostic (the Redis "_job" list).
	if _, err := s.q.Exec(ctx, insertDiagnosticSQL, event.JobID, nullIfEmpty(event.NodeID), serialized); err != nil {
		return fmt.Errorf("failed to insert diagnostic: %w", err)
	}
	return nil
}

func (s *Storage) SaveJobComplete(ctx context.Context, event *job.JobCompleteEvent) error {
	if event == nil {
		return errors.New("cannot save nil job complete event")
	}
	data, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal job complete event: %w", err)
	}
	if _, err := s.q.Exec(ctx, upsertJobCompleteSQL, event.JobID, data); err != nil {
		return fmt.Errorf("failed to save job complete event: %w", err)
	}
	return nil
}

// requiredPrivileges covers the writes and the janitor; checked one by one since has_table_privilege matches ANY listed.
var requiredPrivileges = []string{"SELECT", "INSERT", "UPDATE", "DELETE"}

// readySQL lists live tables that are missing or lack a required privilege for the current user.
const readySQL = `SELECT coalesce(array_agg(DISTINCT problem ORDER BY problem), '{}'::text[]) FROM (
  SELECT CASE WHEN to_regclass(t) IS NULL THEN t || ': missing'
              ELSE t || ': no ' || p || ' privilege' END AS problem
  FROM unnest($1::text[]) AS t CROSS JOIN unnest($2::text[]) AS p
  WHERE to_regclass(t) IS NULL OR NOT has_table_privilege(to_regclass(t), p)
) s`

// Ping reports whether the live tables exist and the current user can write and reap them.
func (s *Storage) Ping(ctx context.Context) error {
	var problems []string
	if err := s.q.QueryRow(ctx, readySQL, liveTables(), requiredPrivileges).Scan(&problems); err != nil {
		return fmt.Errorf("check live tables: %w", err)
	}
	if len(problems) > 0 {
		return fmt.Errorf("live tables not ready: %s", strings.Join(problems, "; "))
	}
	return nil
}

func liveTables() []string {
	tables := make([]string, len(reapTargets))
	for i, t := range reapTargets {
		tables[i] = t.table
	}
	return tables
}

func nullIfEmpty(s *string) *string {
	if s == nil || *s == "" {
		return nil
	}
	return s
}
