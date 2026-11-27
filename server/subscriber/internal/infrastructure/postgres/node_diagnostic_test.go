package postgres

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

func newDiagnosticEvent(jobID string, nodeID *string) *diagnostic.DiagnosticEvent {
	length := uint(3)
	return &diagnostic.DiagnosticEvent{
		Timestamp:  time.Date(2026, 1, 1, 12, 0, 0, 123456000, time.UTC),
		WorkflowID: testWorkflowID,
		JobID:      jobID,
		Schema:     diagnostic.DiagnosticSchemaV1,
		WireDiagnostic: diagnostic.WireDiagnostic{
			NodeID:               nodeID,
			EffectiveDisposition: strp("warn"),
			ActionType:           strp("processor"),
			FeatureID:            strp("feature-1"),
			Help:                 strp("check the input"),
			SourceSpan:           &diagnostic.WireSourceSpan{Offset: 7, Length: &length},
			Aggregated:           &diagnostic.WireAggregateInfo{Count: 4, SampleFeatureIds: []string{"f1", "f2"}},
			Code:                 "geometry.invalid",
			Category:             "geometry",
			Severity:             "warn",
			Message:              "bad ring\x00",
		},
	}
}

func TestNodeDiagnostics_SaveDurableDiagnostic(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()
	s := NewNodeDiagnostics(pool)

	event := newDiagnosticEvent("job-1", strp(testNodeID1))
	require.NoError(t, s.SaveDurableDiagnostic(ctx, event))

	var (
		id, workflowID, schema, code, category, severity string
		nodeID, disposition, actionType, featureID, help *string
		message                                          []byte
		ts                                               time.Time
		spanOffset, spanLength, aggCount                 *int64
		sampleIDs                                        []string
	)
	require.NoError(t, pool.QueryRow(ctx, `SELECT id, workflow_id, node_id, ts, schema, code, category, severity, message,
		effective_disposition, action_type, feature_id, help,
		source_span_offset, source_span_length, aggregated_count, aggregated_sample_feature_ids
		FROM node_diagnostics WHERE job_id = 'job-1'`,
	).Scan(&id, &workflowID, &nodeID, &ts, &schema, &code, &category, &severity, &message,
		&disposition, &actionType, &featureID, &help,
		&spanOffset, &spanLength, &aggCount, &sampleIDs))

	assert.Contains(t, id, "job-1:"+testNodeID1+":")
	assert.Equal(t, testWorkflowID, workflowID)
	assert.Equal(t, strp(testNodeID1), nodeID)
	assert.True(t, event.Timestamp.Equal(ts))
	assert.Equal(t, diagnostic.DiagnosticSchemaV1, schema)
	assert.Equal(t, "geometry.invalid", code)
	assert.Equal(t, "geometry", category)
	assert.Equal(t, "warn", severity)
	assert.Equal(t, "bad ring\x00", string(message))
	assert.Equal(t, strp("warn"), disposition)
	assert.Equal(t, strp("processor"), actionType)
	assert.Equal(t, strp("feature-1"), featureID)
	assert.Equal(t, strp("check the input"), help)
	require.NotNil(t, spanOffset)
	assert.Equal(t, int64(7), *spanOffset)
	require.NotNil(t, spanLength)
	assert.Equal(t, int64(3), *spanLength)
	require.NotNil(t, aggCount)
	assert.Equal(t, int64(4), *aggCount)
	assert.Equal(t, []string{"f1", "f2"}, sampleIDs)
}

func TestNodeDiagnostics_SaveDurableDiagnostic_RedeliveryIsDeduplicated(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()
	s := NewNodeDiagnostics(pool)

	require.NoError(t, s.SaveDurableDiagnostic(ctx, newDiagnosticEvent("job-dup", strp(testNodeID1))))
	require.NoError(t, s.SaveDurableDiagnostic(ctx, newDiagnosticEvent("job-dup", strp(testNodeID1))))
	assert.Equal(t, 1, count(t, pool, `SELECT count(*) FROM node_diagnostics WHERE job_id = 'job-dup'`))

	other := newDiagnosticEvent("job-dup", strp(testNodeID1))
	other.Message = "another message"
	require.NoError(t, s.SaveDurableDiagnostic(ctx, other))
	assert.Equal(t, 2, count(t, pool, `SELECT count(*) FROM node_diagnostics WHERE job_id = 'job-dup'`))
}

func TestNodeDiagnostics_SaveDurableDiagnostic_JobLevel(t *testing.T) {
	t.Parallel()
	pool := newPool(t)
	ctx := context.Background()
	s := NewNodeDiagnostics(pool)

	event := newDiagnosticEvent("job-level", strp(""))
	event.SourceSpan = nil
	event.Aggregated = nil
	require.NoError(t, s.SaveDurableDiagnostic(ctx, event))

	var (
		id                   string
		nodeID               *string
		spanOffset, aggCount *int64
		sampleIDs            []string
	)
	require.NoError(t, pool.QueryRow(ctx,
		`SELECT id, node_id, source_span_offset, aggregated_count, aggregated_sample_feature_ids FROM node_diagnostics WHERE job_id = 'job-level'`,
	).Scan(&id, &nodeID, &spanOffset, &aggCount, &sampleIDs))
	assert.Contains(t, id, "job-level:_job:")
	assert.Nil(t, nodeID)
	assert.Nil(t, spanOffset)
	assert.Nil(t, aggCount)
	assert.Nil(t, sampleIDs)
}

func TestNodeDiagnostics_SaveDurableDiagnostic_Nil(t *testing.T) {
	t.Parallel()
	assert.Error(t, NewNodeDiagnostics(nil).SaveDurableDiagnostic(context.Background(), nil))
}

func TestNodeDiagnostics_Ping(t *testing.T) {
	t.Parallel()

	t.Run("ready", func(t *testing.T) {
		t.Parallel()
		pool := newPool(t)
		assert.NoError(t, NewNodeDiagnostics(pool).Ping(context.Background()))
	})

	t.Run("missing table", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		pool := newPool(t)
		_, err := pool.Exec(ctx, `DROP TABLE node_diagnostics`)
		require.NoError(t, err)

		err = NewNodeDiagnostics(pool).Ping(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "node_diagnostics: missing")
	})

	t.Run("INSERT without SELECT is rejected", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		pool := newPool(t)
		conn := connAsRole(t, pool, "INSERT")

		err := NewNodeDiagnostics(conn).Ping(ctx)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "node_diagnostics: no SELECT privilege")
		assert.ErrorContains(t, NewNodeDiagnostics(conn).SaveDurableDiagnostic(ctx, newDiagnosticEvent("job-ins", strp(testNodeID1))), "permission denied")
	})

	t.Run("SELECT and INSERT are enough", func(t *testing.T) {
		t.Parallel()
		ctx := context.Background()
		pool := newPool(t)
		conn := connAsRole(t, pool, "SELECT, INSERT")

		require.NoError(t, NewNodeDiagnostics(conn).Ping(ctx))
		require.NoError(t, NewNodeDiagnostics(conn).SaveDurableDiagnostic(ctx, newDiagnosticEvent("job-sel-ins", strp(testNodeID1))))
	})
}

// connAsRole returns a connection switched to a fresh role holding only the given node_diagnostics privileges.
func connAsRole(t *testing.T, pool *pgxpool.Pool, privileges string) *pgx.Conn {
	t.Helper()
	ctx := context.Background()

	// Roles are cluster-wide, so give this one a unique name and drop it.
	role := fmt.Sprintf("node_diag_%d", time.Now().UnixNano())
	_, err := pool.Exec(ctx, "CREATE ROLE "+role+" NOLOGIN")
	require.NoError(t, err)
	t.Cleanup(func() {
		_, _ = pool.Exec(context.Background(), "DROP OWNED BY "+role)
		_, _ = pool.Exec(context.Background(), "DROP ROLE "+role)
	})
	_, err = pool.Exec(ctx, "GRANT "+privileges+" ON node_diagnostics TO "+role)
	require.NoError(t, err)

	conn, err := pool.Acquire(ctx)
	require.NoError(t, err)
	t.Cleanup(func() {
		_, _ = conn.Exec(context.Background(), "RESET ROLE")
		conn.Release()
	})
	_, err = conn.Exec(ctx, "SET ROLE "+role)
	require.NoError(t, err)
	return conn.Conn()
}
