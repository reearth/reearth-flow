package postgres_test

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newEdgeExecution(eid graph.EdgeExecutionID, edgeID string, jobID id.JobID, url *string) *graph.EdgeExecution {
	e, err := graph.NewEdgeExecutionBuilder().
		ID(eid).
		EdgeID(edgeID).
		JobID(jobID).
		IntermediateDataURL(url).
		Build()
	if err != nil {
		panic(err)
	}
	return e
}

func TestEdgeExecution_Save_FindByJobEdgeID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewEdgeExecution(pgxx.NewClient(pool))

	eid := id.NewEdgeExecutionID()
	jid := id.NewJobID()
	url := "https://example.com/data"
	e := newEdgeExecution(eid, "edge-1", jid, &url)

	require.NoError(t, r.Save(ctx, e))

	got, err := r.FindByJobEdgeID(ctx, jid, "edge-1")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, eid, got.ID())
	assert.Equal(t, "edge-1", got.EdgeID())
	assert.Equal(t, jid, got.JobID())
	require.NotNil(t, got.IntermediateDataURL())
	assert.Equal(t, "https://example.com/data", *got.IntermediateDataURL())
}

func TestEdgeExecution_FindByJobEdgeID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewEdgeExecution(pgxx.NewClient(pool))

	got, err := r.FindByJobEdgeID(ctx, id.NewJobID(), "no-such-edge")
	assert.Nil(t, got)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestEdgeExecution_Save_NilURL(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewEdgeExecution(pgxx.NewClient(pool))

	eid := id.NewEdgeExecutionID()
	jid := id.NewJobID()
	e := newEdgeExecution(eid, "edge-nil", jid, nil)

	require.NoError(t, r.Save(ctx, e))

	got, err := r.FindByJobEdgeID(ctx, jid, "edge-nil")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Nil(t, got.IntermediateDataURL())
}

func TestEdgeExecution_Save_Upsert(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewEdgeExecution(pgxx.NewClient(pool))

	eid := id.NewEdgeExecutionID()
	jid := id.NewJobID()
	url1 := "https://example.com/v1"
	e := newEdgeExecution(eid, "edge-upsert", jid, &url1)
	require.NoError(t, r.Save(ctx, e))

	// Save again with updated URL (same ID = upsert).
	url2 := "https://example.com/v2"
	e2 := newEdgeExecution(eid, "edge-upsert", jid, &url2)
	require.NoError(t, r.Save(ctx, e2))

	got, err := r.FindByJobEdgeID(ctx, jid, "edge-upsert")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, "https://example.com/v2", *got.IntermediateDataURL())
}

func TestEdgeExecution_FindByJobEdgeID_MultipleEdges(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewEdgeExecution(pgxx.NewClient(pool))

	jid := id.NewJobID()
	e1 := newEdgeExecution(id.NewEdgeExecutionID(), "edge-A", jid, nil)
	e2 := newEdgeExecution(id.NewEdgeExecutionID(), "edge-B", jid, nil)
	require.NoError(t, r.Save(ctx, e1))
	require.NoError(t, r.Save(ctx, e2))

	gotA, err := r.FindByJobEdgeID(ctx, jid, "edge-A")
	require.NoError(t, err)
	assert.Equal(t, "edge-A", gotA.EdgeID())

	gotB, err := r.FindByJobEdgeID(ctx, jid, "edge-B")
	require.NoError(t, err)
	assert.Equal(t, "edge-B", gotB.EdgeID())
}
