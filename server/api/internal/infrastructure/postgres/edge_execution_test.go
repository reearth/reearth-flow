package postgres_test

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
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

func TestEdgeExecution_Save(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	client := pgxx.NewClient(pool)
	r := postgres.NewEdgeExecution(client)

	eid := id.NewEdgeExecutionID()
	jid := id.NewJobID()
	url := "https://example.com/data"
	e := newEdgeExecution(eid, "edge-1", jid, &url)

	require.NoError(t, r.Save(ctx, e))

	got, err := gen.New(pool).GetEdgeExecutionByJobEdgeID(ctx, gen.GetEdgeExecutionByJobEdgeIDParams{
		JobID:  jid.String(),
		EdgeID: "edge-1",
	})
	require.NoError(t, err)
	assert.Equal(t, eid.String(), got.ID)
	assert.Equal(t, "edge-1", got.EdgeID)
	assert.Equal(t, jid.String(), got.JobID)
	require.NotNil(t, got.IntermediateDataUrl)
	assert.Equal(t, "https://example.com/data", *got.IntermediateDataUrl)
}

func TestEdgeExecution_Save_NilURL(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewEdgeExecution(pgxx.NewClient(pool))

	eid := id.NewEdgeExecutionID()
	jid := id.NewJobID()
	e := newEdgeExecution(eid, "edge-nil", jid, nil)

	require.NoError(t, r.Save(ctx, e))

	got, err := gen.New(pool).GetEdgeExecutionByJobEdgeID(ctx, gen.GetEdgeExecutionByJobEdgeIDParams{
		JobID:  jid.String(),
		EdgeID: "edge-nil",
	})
	require.NoError(t, err)
	assert.Nil(t, got.IntermediateDataUrl)
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

	url2 := "https://example.com/v2"
	e2 := newEdgeExecution(eid, "edge-upsert", jid, &url2)
	require.NoError(t, r.Save(ctx, e2))

	got, err := gen.New(pool).GetEdgeExecutionByJobEdgeID(ctx, gen.GetEdgeExecutionByJobEdgeIDParams{
		JobID:  jid.String(),
		EdgeID: "edge-upsert",
	})
	require.NoError(t, err)
	require.NotNil(t, got.IntermediateDataUrl)
	assert.Equal(t, "https://example.com/v2", *got.IntermediateDataUrl)
}
