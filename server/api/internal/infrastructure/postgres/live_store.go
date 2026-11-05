package postgres

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	reearth_log "github.com/reearth/reearthx/log"
)

// LiveStore reads the transient job data the subscriber writes to the unlogged
// job_* tables when REEARTH_FLOW_LIVE_BACKEND=postgres. It is the Postgres
// counterpart of the Redis gateway and returns the same shapes.
type LiveStore struct {
	pool *pgxpool.Pool
	q    *gen.Queries
}

var (
	_ gateway.LiveStore       = (*LiveStore)(nil)
	_ gateway.LogsBatchReader = (*LiveStore)(nil)
)

// NewLiveStore builds a LiveStore over pool. No transaction is ever needed:
// every read is a single statement.
func NewLiveStore(pool *pgxpool.Pool) *LiveStore {
	return &LiveStore{pool: pool, q: gen.New(pool)}
}

// Ping reports whether the database is reachable, for the health check.
func (s *LiveStore) Ping(ctx context.Context) error {
	return s.pool.Ping(ctx)
}

func (s *LiveStore) GetLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*log.Log, error) {
	rows, err := s.q.ListJobLogs(ctx, gen.ListJobLogsParams{
		JobID: jobID.String(),
		Since: since.UTC(),
		Until: until.UTC(),
	})
	if err != nil {
		return nil, fmt.Errorf("failed to list job logs: %w", err)
	}
	var result []*log.Log
	for _, r := range rows {
		result = append(result, toLog(ctx, jobID, r.NodeID, r.Ts, r.Level, r.Message))
	}
	return result, nil
}

func (s *LiveStore) GetLogsBatch(ctx context.Context, since, until time.Time, jobIDs []id.JobID) (map[id.JobID][]*log.Log, error) {
	result := make(map[id.JobID][]*log.Log, len(jobIDs))
	if len(jobIDs) == 0 {
		return result, nil
	}
	byString := make(map[string]id.JobID, len(jobIDs))
	keys := make([]string, 0, len(jobIDs))
	for _, jid := range jobIDs {
		byString[jid.String()] = jid
		keys = append(keys, jid.String())
	}

	rows, err := s.q.ListJobLogsByJobIDs(ctx, gen.ListJobLogsByJobIDsParams{
		JobIds: keys,
		Since:  since.UTC(),
		Until:  until.UTC(),
	})
	if err != nil {
		return nil, fmt.Errorf("failed to list job logs: %w", err)
	}
	for _, r := range rows {
		jid, ok := byString[r.JobID]
		if !ok {
			continue
		}
		result[jid] = append(result[jid], toLog(ctx, jid, r.NodeID, r.Ts, r.Level, r.Message))
	}
	return result, nil
}

func toLog(ctx context.Context, jobID id.JobID, nodeID *string, ts time.Time, level, message string) *log.Log {
	var nid *log.NodeID
	if nodeID != nil && *nodeID != "" {
		n, err := id.NodeIDFrom(*nodeID)
		if err != nil {
			reearth_log.Warnfc(ctx, "postgres: invalid node ID in log entry: %v", *nodeID)
		} else {
			nid = &n
		}
	}
	return log.NewLog(jobID, nid, ts.UTC(), log.Level(level), message)
}

func (s *LiveStore) GetUserFacingLogs(ctx context.Context, since, until time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error) {
	rows, err := s.q.ListJobUserFacingLogs(ctx, gen.ListJobUserFacingLogsParams{
		JobID: jobID.String(),
		Since: since.UTC(),
		Until: until.UTC(),
	})
	if err != nil {
		return nil, fmt.Errorf("failed to list user-facing logs: %w", err)
	}
	var result []*userfacinglog.UserFacingLog
	for _, r := range rows {
		result = append(result, userfacinglog.NewUserFacingLogWithDetails(
			jobID,
			r.Ts.UTC(),
			toUserFacingLogLevel(r.Level),
			r.NodeID,
			r.NodeName,
			r.Message,
			json.RawMessage(r.Metadata),
		))
	}
	return result, nil
}

// toUserFacingLogLevel mirrors the Redis reader: unknown levels read as info.
func toUserFacingLogLevel(level string) userfacinglog.LogLevel {
	switch level {
	case "success", "SUCCESS":
		return userfacinglog.LogLevelSuccess
	case "error", "ERROR":
		return userfacinglog.LogLevelError
	default:
		return userfacinglog.LogLevelInfo
	}
}

func (s *LiveStore) GetJobCompleteEvent(ctx context.Context, jobID id.JobID) (*gateway.JobCompleteEvent, error) {
	data, err := s.q.GetJobCompleteEvent(ctx, jobID.String())
	if errors.Is(err, pgx.ErrNoRows) {
		// The worker hasn't reported yet.
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get job complete event: %w", err)
	}
	var event gateway.JobCompleteEvent
	if err := json.Unmarshal(data, &event); err != nil {
		return nil, fmt.Errorf("failed to unmarshal job complete event: %w", err)
	}
	return &event, nil
}

func (s *LiveStore) DeleteJobCompleteEvent(ctx context.Context, jobID id.JobID) error {
	if err := s.q.DeleteJobCompleteEvent(ctx, jobID.String()); err != nil {
		return fmt.Errorf("failed to delete job complete event: %w", err)
	}
	return nil
}

func (s *LiveStore) GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	var (
		rows [][]byte
		err  error
	)
	// An empty node ID addresses the job-level diagnostics (Redis' "_job" list).
	if nodeID == "" {
		rows, err = s.q.ListJobLevelDiagnostics(ctx, jobID.String())
	} else {
		rows, err = s.q.ListNodeDiagnostics(ctx, gen.ListNodeDiagnosticsParams{JobID: jobID.String(), NodeID: &nodeID})
	}
	if err != nil {
		return nil, fmt.Errorf("failed to list node diagnostics: %w", err)
	}
	return toDiagnostics(ctx, rows), nil
}

func (s *LiveStore) GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	rows, err := s.q.ListJobDiagnostics(ctx, jobID.String())
	if err != nil {
		return nil, fmt.Errorf("failed to list job diagnostics: %w", err)
	}
	return toDiagnostics(ctx, rows), nil
}

// diagnosticEntry is the subscriber's serialized DiagnosticEvent; WireDiagnostic
// is embedded, not nested, to match its flat JSON shape.
type diagnosticEntry struct {
	Timestamp time.Time `json:"timestamp"`
	JobID     string    `json:"jobId"`
	gateway.WireDiagnostic
}

// toDiagnostics skips undecodable rows, as the Redis reader does, so one bad
// event doesn't hide the rest of a job's diagnostics.
func toDiagnostics(ctx context.Context, rows [][]byte) []*diagnostic.Diagnostic {
	var result []*diagnostic.Diagnostic
	for _, data := range rows {
		var entry diagnosticEntry
		if err := json.Unmarshal(data, &entry); err != nil {
			reearth_log.Warnfc(ctx, "postgres: failed to unmarshal diagnostic entry: %s", data)
			continue
		}
		jid, err := id.JobIDFrom(entry.JobID)
		if err != nil {
			reearth_log.Warnfc(ctx, "postgres: invalid job ID in diagnostic entry: %v", err)
			continue
		}
		d, err := entry.ToDomain(jid, entry.Timestamp.UTC())
		if err != nil {
			reearth_log.Warnfc(ctx, "postgres: failed to convert diagnostic entry to domain: %v", err)
			continue
		}
		result = append(result, d)
	}
	return result
}
