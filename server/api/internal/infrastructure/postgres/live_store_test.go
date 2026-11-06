package postgres_test

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// The subscriber owns the writes; these helpers insert rows the way it does so
// the reader is tested against the same shapes.

func insertJobLog(t *testing.T, pool *pgxpool.Pool, jobID id.JobID, nodeID *string, ts time.Time, level, msg string) {
	t.Helper()
	_, err := pool.Exec(context.Background(),
		`INSERT INTO job_logs (job_id, workflow_id, node_id, ts, level, message, payload_hash)
		 VALUES ($1, 'wf', $2, $3, $4, $5, $6)`,
		jobID.String(), nodeID, ts, level, msg, []byte(fmt.Sprintf("%s-%d", msg, time.Now().UnixNano())))
	require.NoError(t, err)
}

func insertUserFacingLog(t *testing.T, pool *pgxpool.Pool, jobID id.JobID, nodeID, nodeName *string, ts time.Time, level, msg string, metadata []byte) {
	t.Helper()
	_, err := pool.Exec(context.Background(),
		`INSERT INTO job_user_facing_logs (job_id, workflow_id, node_id, node_name, ts, level, message, metadata, payload_hash)
		 VALUES ($1, 'wf', $2, $3, $4, $5, $6, $7, $8)`,
		jobID.String(), nodeID, nodeName, ts, level, msg, metadata, []byte(fmt.Sprintf("%s-%d", msg, time.Now().UnixNano())))
	require.NoError(t, err)
}

func insertDiagnostic(t *testing.T, pool *pgxpool.Pool, jobID id.JobID, nodeID *string, code string) {
	t.Helper()
	data := map[string]any{
		"timestamp":  time.Now().UTC(),
		"workflowId": "wf",
		"jobId":      jobID.String(),
		"schema":     "diagnostic.v1",
		"code":       code,
		"category":   "data",
		"severity":   "warning",
		"message":    "msg " + code,
	}
	if nodeID != nil {
		data["nodeId"] = *nodeID
	}
	b, err := json.Marshal(data)
	require.NoError(t, err)
	_, err = pool.Exec(context.Background(),
		`INSERT INTO job_diagnostics (job_id, node_id, data) VALUES ($1, $2, $3)`,
		jobID.String(), nodeID, b)
	require.NoError(t, err)
}

func strp(s string) *string { return &s }

func TestLiveStore_TablesAreUnlogged(t *testing.T) {
	t.Parallel()
	pool := pgtest.Connect(t)(t)

	for _, table := range []string{"job_logs", "job_user_facing_logs", "job_diagnostics", "job_complete_events"} {
		var persistence string
		require.NoError(t, pool.QueryRow(context.Background(),
			`SELECT relpersistence::text FROM pg_class WHERE relname = $1`, table).Scan(&persistence))
		assert.Equal(t, "u", persistence, "%s must be UNLOGGED", table)
	}
}

func TestLiveStore_GetLogs(t *testing.T) {
	t.Parallel()
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	s := postgres.NewLiveStore(pool)

	jid, other := id.NewJobID(), id.NewJobID()
	nid := id.NewNodeID()
	base := time.Date(2026, 1, 1, 12, 0, 0, 0, time.UTC)

	// Inserted out of order: the reader must sort by ts.
	insertJobLog(t, pool, jid, nil, base.Add(2*time.Second), "INFO", "second")
	insertJobLog(t, pool, jid, strp(nid.String()), base.Add(time.Second), "WARN", "first")
	insertJobLog(t, pool, jid, nil, base.Add(-time.Minute), "INFO", "before window")
	insertJobLog(t, pool, jid, nil, base.Add(time.Hour), "INFO", "after window")
	insertJobLog(t, pool, jid, strp("not-a-node-id"), base.Add(3*time.Second), "ERROR", "bad node")
	insertJobLog(t, pool, other, nil, base.Add(time.Second), "INFO", "other job")

	got, err := s.GetLogs(ctx, base, base.Add(time.Minute), jid)
	require.NoError(t, err)
	require.Len(t, got, 3)

	assert.Equal(t, "first", got[0].Message())
	assert.Equal(t, jid, got[0].JobID())
	assert.Equal(t, "WARN", string(got[0].Level()))
	require.NotNil(t, got[0].NodeID())
	assert.Equal(t, nid, *got[0].NodeID())
	assert.True(t, base.Add(time.Second).Equal(got[0].Timestamp()))

	assert.Equal(t, "second", got[1].Message())
	assert.Nil(t, got[1].NodeID())

	// An unparseable node ID is dropped, not fatal, as in the Redis reader.
	assert.Equal(t, "bad node", got[2].Message())
	assert.Nil(t, got[2].NodeID())

	t.Run("no rows", func(t *testing.T) {
		t.Parallel()
		got, err := s.GetLogs(ctx, base, base.Add(time.Minute), id.NewJobID())
		require.NoError(t, err)
		assert.Empty(t, got)
	})
}

func TestLiveStore_GetLogsBatch(t *testing.T) {
	t.Parallel()
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	s := postgres.NewLiveStore(pool)

	a, b, empty, unasked := id.NewJobID(), id.NewJobID(), id.NewJobID(), id.NewJobID()
	base := time.Date(2026, 1, 1, 12, 0, 0, 0, time.UTC)
	insertJobLog(t, pool, a, nil, base.Add(2*time.Second), "INFO", "a2")
	insertJobLog(t, pool, a, nil, base.Add(time.Second), "INFO", "a1")
	insertJobLog(t, pool, b, nil, base.Add(time.Second), "INFO", "b1")
	insertJobLog(t, pool, b, nil, base.Add(-time.Second), "INFO", "b before window")
	insertJobLog(t, pool, unasked, nil, base.Add(time.Second), "INFO", "not requested")

	got, err := s.GetLogsBatch(ctx, base, base.Add(time.Minute), []id.JobID{a, b, empty})
	require.NoError(t, err)
	require.Len(t, got[a], 2)
	assert.Equal(t, "a1", got[a][0].Message())
	assert.Equal(t, "a2", got[a][1].Message())
	require.Len(t, got[b], 1)
	assert.Equal(t, "b1", got[b][0].Message())
	assert.Empty(t, got[empty])
	assert.NotContains(t, got, unasked)

	t.Run("no job IDs", func(t *testing.T) {
		t.Parallel()
		got, err := s.GetLogsBatch(ctx, base, base.Add(time.Minute), nil)
		require.NoError(t, err)
		assert.Empty(t, got)
	})
}

func TestLiveStore_GetUserFacingLogs(t *testing.T) {
	t.Parallel()
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	s := postgres.NewLiveStore(pool)

	jid := id.NewJobID()
	base := time.Date(2026, 1, 1, 12, 0, 0, 0, time.UTC)
	insertUserFacingLog(t, pool, jid, strp("node-1"), strp("Reader"), base.Add(2*time.Second), "SUCCESS", "done", []byte(`{"rows":3}`))
	insertUserFacingLog(t, pool, jid, nil, nil, base.Add(time.Second), "ERROR", "failed", nil)
	insertUserFacingLog(t, pool, jid, nil, nil, base.Add(3*time.Second), "SOMETHING", "unknown level", nil)
	insertUserFacingLog(t, pool, jid, nil, nil, base.Add(-time.Second), "INFO", "before window", nil)

	got, err := s.GetUserFacingLogs(ctx, base, base.Add(time.Minute), jid)
	require.NoError(t, err)
	require.Len(t, got, 3)

	assert.Equal(t, "failed", got[0].Message())
	assert.Equal(t, userfacinglog.LogLevelError, got[0].Level())
	assert.Nil(t, got[0].NodeID())
	assert.Nil(t, got[0].NodeName())
	assert.Empty(t, got[0].Metadata())

	assert.Equal(t, "done", got[1].Message())
	assert.Equal(t, userfacinglog.LogLevelSuccess, got[1].Level())
	assert.Equal(t, strp("node-1"), got[1].NodeID())
	assert.Equal(t, strp("Reader"), got[1].NodeName())
	assert.JSONEq(t, `{"rows":3}`, string(got[1].Metadata()))

	assert.Equal(t, userfacinglog.LogLevelInfo, got[2].Level())
}

func TestLiveStore_Diagnostics(t *testing.T) {
	t.Parallel()
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	s := postgres.NewLiveStore(pool)

	jid := id.NewJobID()
	insertDiagnostic(t, pool, jid, strp("node-1"), "N1-old")
	insertDiagnostic(t, pool, jid, nil, "JOB")
	insertDiagnostic(t, pool, jid, strp("node-2"), "N2")
	insertDiagnostic(t, pool, jid, strp("node-1"), "N1-new")
	insertDiagnostic(t, pool, id.NewJobID(), strp("node-1"), "other job")
	_, err := pool.Exec(ctx, `INSERT INTO job_diagnostics (job_id, node_id, data) VALUES ($1, 'node-1', '{"jobId": "not-a-job-id"}')`, jid.String())
	require.NoError(t, err)

	t.Run("node diagnostics newest first, bad rows skipped", func(t *testing.T) {
		t.Parallel()
		got, err := s.GetNodeDiagnostics(ctx, jid, "node-1")
		require.NoError(t, err)
		require.Len(t, got, 2)
		assert.Equal(t, "N1-new", got[0].Code())
		assert.Equal(t, "N1-old", got[1].Code())
		assert.Equal(t, jid, got[0].JobID())
	})

	t.Run("empty node ID reads job-level diagnostics", func(t *testing.T) {
		t.Parallel()
		got, err := s.GetNodeDiagnostics(ctx, jid, "")
		require.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "JOB", got[0].Code())
	})

	t.Run("job diagnostics span every node, newest first", func(t *testing.T) {
		t.Parallel()
		got, err := s.GetJobDiagnostics(ctx, jid)
		require.NoError(t, err)
		var out []string
		for _, d := range got {
			out = append(out, d.Code())
		}
		assert.Equal(t, []string{"N1-new", "N2", "JOB", "N1-old"}, out)
	})

	t.Run("missing job is nil, nil", func(t *testing.T) {
		t.Parallel()
		got, err := s.GetJobDiagnostics(ctx, id.NewJobID())
		require.NoError(t, err)
		assert.Nil(t, got)
	})
}

func TestLiveStore_JobCompleteEvent(t *testing.T) {
	t.Parallel()
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	s := postgres.NewLiveStore(pool)

	jid := id.NewJobID()

	got, err := s.GetJobCompleteEvent(ctx, jid)
	require.NoError(t, err)
	assert.Nil(t, got, "a job the worker hasn't reported is nil, not an error")

	_, err = pool.Exec(ctx, `INSERT INTO job_complete_events (job_id, data) VALUES ($1, $2)`, jid.String(),
		[]byte(fmt.Sprintf(`{"timestamp":"2026-01-01T12:00:00Z","workflowId":"wf","jobId":%q,"result":"success","failedNodes":[{"code":"E1","category":"c","severity":"error","message":"m"}]}`, jid.String())))
	require.NoError(t, err)

	got, err = s.GetJobCompleteEvent(ctx, jid)
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, jid.String(), got.JobID)
	assert.Equal(t, "success", got.Result)
	require.Len(t, got.FailedNodes, 1)
	assert.Equal(t, "E1", got.FailedNodes[0].Code)

	require.NoError(t, s.DeleteJobCompleteEvent(ctx, jid))
	got, err = s.GetJobCompleteEvent(ctx, jid)
	require.NoError(t, err)
	assert.Nil(t, got)

	// Deleting an absent event is not an error, matching Redis DEL.
	require.NoError(t, s.DeleteJobCompleteEvent(ctx, jid))
}

func TestLiveStore_Ping(t *testing.T) {
	t.Parallel()
	pool := pgtest.Connect(t)(t)
	assert.NoError(t, postgres.NewLiveStore(pool).Ping(context.Background()))
}
