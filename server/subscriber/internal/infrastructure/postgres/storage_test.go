package postgres

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

func strp(s string) *string { return &s }

func TestStorage_SaveLog(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()
	s := NewStorage(pool)

	ts := time.Date(2026, 1, 1, 12, 0, 0, 123456000, time.UTC)
	event := &domainLog.LogEvent{
		WorkflowID: testWorkflowID,
		JobID:      "job-1",
		NodeID:     strp(testNodeID1),
		Timestamp:  ts,
		LogLevel:   domainLog.LogLevelWarn,
		Message:    "hello",
	}
	require.NoError(t, s.SaveLog(ctx, event))

	var (
		workflowID, level string
		message           []byte
		nodeID            *string
		gotTS             time.Time
		hash              []byte
	)
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT workflow_id, node_id, ts, level, message, payload_hash FROM job_logs WHERE job_id = 'job-1'`,
	).Scan(&workflowID, &nodeID, &gotTS, &level, &message, &hash))
	assert.Equal(t, testWorkflowID, workflowID)
	assert.Equal(t, strp(testNodeID1), nodeID)
	assert.True(t, ts.Equal(gotTS))
	assert.Equal(t, "WARN", level)
	assert.Equal(t, "hello", string(message))
	assert.Len(t, hash, 32)

	t.Run("redelivery is deduplicated", func(t *testing.T) {
		t.Parallel()
		dup := *event
		require.NoError(t, s.SaveLog(ctx, &dup))
		assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_logs WHERE job_id = 'job-1'`))
	})

	t.Run("a different line is kept", func(t *testing.T) {
		t.Parallel()
		other := *event
		other.JobID = "job-1-other"
		other.Message = "world"
		require.NoError(t, s.SaveLog(ctx, &other))
		assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_logs WHERE job_id = 'job-1-other'`))
	})

	t.Run("empty node ID is stored as NULL", func(t *testing.T) {
		t.Parallel()
		e := *event
		e.JobID = "job-1-empty-node"
		e.NodeID = strp("")
		require.NoError(t, s.SaveLog(ctx, &e))
		assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_logs WHERE job_id = 'job-1-empty-node' AND node_id IS NULL`))
	})

	t.Run("nil event", func(t *testing.T) {
		t.Parallel()
		assert.Error(t, s.SaveLog(ctx, nil))
	})
}

func TestStorage_SaveUserFacingLog(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()
	s := NewStorage(pool)

	event := &userfacinglog.UserFacingLogEvent{
		WorkflowID: testWorkflowID,
		JobID:      "job-1",
		Timestamp:  time.Date(2026, 1, 1, 12, 0, 0, 0, time.UTC),
		Level:      userfacinglog.UserFacingLogLevelSuccess,
		NodeName:   strp("Reader"),
		NodeID:     strp(testNodeID1),
		Message:    "done",
	}
	require.NoError(t, s.SaveUserFacingLog(ctx, event))
	require.NoError(t, s.SaveUserFacingLog(ctx, event), "redelivery must not fail")

	var nodeName, nodeID *string
	var level string
	var message []byte
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT node_id, node_name, level, message FROM job_user_facing_logs WHERE job_id = 'job-1'`,
	).Scan(&nodeID, &nodeName, &level, &message))
	assert.Equal(t, strp(testNodeID1), nodeID)
	assert.Equal(t, strp("Reader"), nodeName)
	assert.Equal(t, "SUCCESS", level)
	assert.Equal(t, "done", string(message))
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_user_facing_logs WHERE job_id = 'job-1'`))
}

func TestStorage_SaveDiagnostic(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()
	s := NewStorage(pool)

	newEvent := func(nodeID *string, code string) *diagnostic.DiagnosticEvent {
		return &diagnostic.DiagnosticEvent{
			Timestamp:  time.Date(2026, 1, 1, 12, 0, 0, 0, time.UTC),
			WorkflowID: testWorkflowID,
			JobID:      "job-1",
			Schema:     diagnostic.DiagnosticSchemaV1,
			WireDiagnostic: diagnostic.WireDiagnostic{
				NodeID:   nodeID,
				Code:     code,
				Category: "data",
				Severity: "warning",
				Message:  "m",
			},
		}
	}

	require.NoError(t, s.SaveDiagnostic(ctx, newEvent(strp(testNodeID1), "A")))
	require.NoError(t, s.SaveDiagnostic(ctx, newEvent(nil, "JOB")))
	require.NoError(t, s.SaveDiagnostic(ctx, newEvent(strp(""), "JOB-EMPTY")))
	// Diagnostics append, like LPUSH: the same event twice is two rows.
	require.NoError(t, s.SaveDiagnostic(ctx, newEvent(strp(testNodeID1), "A")))

	assert.Equal(t, 2, count(t, pool, `SELECT count(*) FROM job_diagnostics WHERE job_id = 'job-1' AND node_id = '7e1d2c3b-4a59-4687-b9a0-c1d2e3f4a5b6'`))
	assert.Equal(t, 2, count(t, pool, `SELECT count(*) FROM job_diagnostics WHERE job_id = 'job-1' AND node_id IS NULL`))

	var data []byte
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT data FROM job_diagnostics WHERE job_id = 'job-1' AND node_id = '7e1d2c3b-4a59-4687-b9a0-c1d2e3f4a5b6' ORDER BY id LIMIT 1`).Scan(&data))
	var round diagnostic.DiagnosticEvent
	require.NoError(t, json.Unmarshal(data, &round))
	assert.Equal(t, "A", round.Code)
	assert.Equal(t, "job-1", round.JobID)
	assert.Equal(t, diagnostic.DiagnosticSchemaV1, round.Schema)

	assert.Error(t, s.SaveDiagnostic(ctx, nil))
}

func TestStorage_SaveJobComplete(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()
	s := NewStorage(pool)

	event := &job.JobCompleteEvent{
		Timestamp:  time.Date(2026, 1, 1, 12, 0, 0, 0, time.UTC),
		WorkflowID: testWorkflowID,
		JobID:      "job-1",
		Result:     "failed",
	}
	require.NoError(t, s.SaveJobComplete(ctx, event))

	// Backdate it, then save again: the upsert replaces the data and restarts
	// retention, as a Redis SET with a TTL would.
	_, err := pool.Exec(ctx, `UPDATE job_complete_events SET created_at = now() - interval '1 day' WHERE job_id = 'job-1'`)
	require.NoError(t, err)
	event.Result = "success"
	require.NoError(t, s.SaveJobComplete(ctx, event))

	var data []byte
	var fresh bool
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT data, created_at > now() - interval '1 minute' FROM job_complete_events WHERE job_id = 'job-1'`).Scan(&data, &fresh))
	var got job.JobCompleteEvent
	require.NoError(t, json.Unmarshal(data, &got))
	assert.Equal(t, "success", got.Result)
	assert.True(t, fresh)
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM job_complete_events`))

	assert.Error(t, s.SaveJobComplete(ctx, nil))
}

func TestStorage_Ping(t *testing.T) {
	t.Parallel()

	t.Run("ready", func(t *testing.T) {
		t.Parallel()
		pool := newPool(t)
		assert.NoError(t, NewStorage(pool).Ping(context.Background()))
	})

	t.Run("missing table", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		pool := newPool(t)
		_, err := pool.Exec(ctx, "DROP TABLE job_complete_events")
		require.NoError(t, err)

		err = NewStorage(pool).Ping(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "job_complete_events: missing")
	})

	t.Run("missing privilege", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		pool := newPool(t)

		// Roles are cluster-wide, so give this one a unique name and drop it.
		role := fmt.Sprintf("live_ro_%d", time.Now().UnixNano())
		_, err := pool.Exec(ctx, "CREATE ROLE "+role+" NOLOGIN")
		require.NoError(t, err)
		t.Cleanup(func() {
			_, _ = pool.Exec(context.Background(), "DROP OWNED BY "+role)
			_, _ = pool.Exec(context.Background(), "DROP ROLE "+role)
		})
		_, err = pool.Exec(ctx, "GRANT SELECT, INSERT, UPDATE, DELETE ON job_logs, job_user_facing_logs, job_diagnostics TO "+role)
		require.NoError(t, err)
		_, err = pool.Exec(ctx, "GRANT SELECT, INSERT ON job_complete_events TO "+role)
		require.NoError(t, err)
		_, err = pool.Exec(ctx, "GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO "+role)
		require.NoError(t, err)

		conn, err := pool.Acquire(ctx)
		require.NoError(t, err)
		defer conn.Release()
		_, err = conn.Exec(ctx, "SET ROLE "+role)
		require.NoError(t, err)
		defer func() { _, _ = conn.Exec(context.Background(), "RESET ROLE") }()

		err = NewStorage(conn.Conn()).Ping(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "job_complete_events: no DELETE privilege")
		assert.Contains(t, err.Error(), "job_complete_events: no UPDATE privilege")
		assert.NotContains(t, err.Error(), "job_logs")
	})

	t.Run("missing sequence privilege", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		pool := newPool(t)

		role := fmt.Sprintf("live_noseq_%d", time.Now().UnixNano())
		_, err := pool.Exec(ctx, "CREATE ROLE "+role+" NOLOGIN")
		require.NoError(t, err)
		t.Cleanup(func() {
			_, _ = pool.Exec(context.Background(), "DROP OWNED BY "+role)
			_, _ = pool.Exec(context.Background(), "DROP ROLE "+role)
		})
		_, err = pool.Exec(ctx, "GRANT SELECT, INSERT, UPDATE, DELETE ON job_logs, job_user_facing_logs, job_diagnostics, job_complete_events TO "+role)
		require.NoError(t, err)

		conn, err := pool.Acquire(ctx)
		require.NoError(t, err)
		defer conn.Release()
		_, err = conn.Exec(ctx, "SET ROLE "+role)
		require.NoError(t, err)
		defer func() { _, _ = conn.Exec(context.Background(), "RESET ROLE") }()

		err = NewStorage(conn.Conn()).Ping(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "job_logs_id_seq: no USAGE or UPDATE privilege")
		assert.Contains(t, err.Error(), "job_user_facing_logs_id_seq: no USAGE or UPDATE privilege")
		assert.Contains(t, err.Error(), "job_diagnostics_id_seq: no USAGE or UPDATE privilege")
		for _, p := range []string{"SELECT", "INSERT", "UPDATE", "DELETE"} {
			assert.NotContains(t, err.Error(), ": no "+p+" privilege", "table privileges are all granted")
		}
	})
}

func TestStorage_NULRoundTrips(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()
	s := NewStorage(pool)
	const msg = "before\x00after"

	require.NoError(t, s.SaveLog(ctx, &domainLog.LogEvent{
		WorkflowID: testWorkflowID, JobID: "job-nul", Timestamp: time.Now().UTC(),
		LogLevel: domainLog.LogLevelInfo, Message: msg,
	}))
	require.NoError(t, s.SaveUserFacingLog(ctx, &userfacinglog.UserFacingLogEvent{
		WorkflowID: testWorkflowID, JobID: "job-nul", Timestamp: time.Now().UTC(),
		Level: userfacinglog.UserFacingLogLevelInfo, Message: msg,
	}))
	require.NoError(t, s.SaveDiagnostic(ctx, &diagnostic.DiagnosticEvent{
		Timestamp: time.Now().UTC(), WorkflowID: testWorkflowID, JobID: "job-nul",
		Schema:         diagnostic.DiagnosticSchemaV1,
		WireDiagnostic: diagnostic.WireDiagnostic{Code: "C", Category: "data", Severity: "warning", Message: msg},
	}))

	var logMsg, uflMsg []byte
	require.NoError(t, pool.QueryRow(ctx, `SELECT message FROM job_logs WHERE job_id = 'job-nul'`).Scan(&logMsg))
	require.NoError(t, pool.QueryRow(ctx, `SELECT message FROM job_user_facing_logs WHERE job_id = 'job-nul'`).Scan(&uflMsg))
	assert.Equal(t, msg, string(logMsg))
	assert.Equal(t, msg, string(uflMsg))

	var data []byte
	require.NoError(t, pool.QueryRow(ctx, `SELECT data FROM job_diagnostics WHERE job_id = 'job-nul'`).Scan(&data))
	var got diagnostic.DiagnosticEvent
	require.NoError(t, json.Unmarshal(data, &got))
	assert.Equal(t, msg, got.Message)
}
