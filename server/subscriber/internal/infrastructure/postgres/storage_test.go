package postgres

import (
	"context"
	"encoding/json"
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
		WorkflowID: "wf-1",
		JobID:      "job-1",
		NodeID:     strp("node-1"),
		Timestamp:  ts,
		LogLevel:   domainLog.LogLevelWarn,
		Message:    "hello",
	}
	require.NoError(t, s.SaveLog(ctx, event))

	var (
		workflowID, level, message string
		nodeID                     *string
		gotTS                      time.Time
		hash                       []byte
	)
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT workflow_id, node_id, ts, level, message, payload_hash FROM job_logs WHERE job_id = 'job-1'`,
	).Scan(&workflowID, &nodeID, &gotTS, &level, &message, &hash))
	assert.Equal(t, "wf-1", workflowID)
	assert.Equal(t, strp("node-1"), nodeID)
	assert.True(t, ts.Equal(gotTS))
	assert.Equal(t, "WARN", level)
	assert.Equal(t, "hello", message)
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
		WorkflowID: "wf-1",
		JobID:      "job-1",
		Timestamp:  time.Date(2026, 1, 1, 12, 0, 0, 0, time.UTC),
		Level:      userfacinglog.UserFacingLogLevelSuccess,
		NodeName:   strp("Reader"),
		NodeID:     strp("node-1"),
		Message:    "done",
	}
	require.NoError(t, s.SaveUserFacingLog(ctx, event))
	require.NoError(t, s.SaveUserFacingLog(ctx, event), "redelivery must not fail")

	var nodeName, nodeID *string
	var level, message string
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT node_id, node_name, level, message FROM job_user_facing_logs WHERE job_id = 'job-1'`,
	).Scan(&nodeID, &nodeName, &level, &message))
	assert.Equal(t, strp("node-1"), nodeID)
	assert.Equal(t, strp("Reader"), nodeName)
	assert.Equal(t, "SUCCESS", level)
	assert.Equal(t, "done", message)
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
			WorkflowID: "wf-1",
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

	require.NoError(t, s.SaveDiagnostic(ctx, newEvent(strp("node-1"), "A")))
	require.NoError(t, s.SaveDiagnostic(ctx, newEvent(nil, "JOB")))
	require.NoError(t, s.SaveDiagnostic(ctx, newEvent(strp(""), "JOB-EMPTY")))
	// Diagnostics append, like LPUSH: the same event twice is two rows.
	require.NoError(t, s.SaveDiagnostic(ctx, newEvent(strp("node-1"), "A")))

	assert.Equal(t, 2, count(t, pool, `SELECT count(*) FROM job_diagnostics WHERE job_id = 'job-1' AND node_id = 'node-1'`))
	assert.Equal(t, 2, count(t, pool, `SELECT count(*) FROM job_diagnostics WHERE job_id = 'job-1' AND node_id IS NULL`))

	var data []byte
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT data FROM job_diagnostics WHERE job_id = 'job-1' AND node_id = 'node-1' ORDER BY id LIMIT 1`).Scan(&data))
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
		WorkflowID: "wf-1",
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
	pool := newPool(t)
	assert.NoError(t, NewStorage(pool).Ping(context.Background()))
}
