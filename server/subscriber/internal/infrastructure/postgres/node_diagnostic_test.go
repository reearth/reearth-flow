package postgres

import (
	"context"
	"testing"
	"time"

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
	pool := newPool(t)
	ctx := context.Background()

	require.NoError(t, NewNodeDiagnostics(pool).Ping(ctx))

	_, err := pool.Exec(ctx, `DROP TABLE node_diagnostics`)
	require.NoError(t, err)
	err = NewNodeDiagnostics(pool).Ping(ctx)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "node_diagnostics: missing")
}
