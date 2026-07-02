package postgres_test

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newNodeExec(execID string, jobID id.JobID, nodeID id.NodeID, status graph.Status, startedAt, completedAt *time.Time) *graph.NodeExecution {
	b := graph.NewNodeExecutionBuilder().
		ID(execID).
		JobID(jobID).
		NodeID(nodeID).
		Status(status)
	if startedAt != nil {
		b = b.StartedAt(startedAt)
	}
	if completedAt != nil {
		b = b.CompletedAt(completedAt)
	}
	e, err := b.Build()
	if err != nil {
		panic(err)
	}
	return e
}

func TestNodeExecution_Save_FindByJobNodeID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeExecution(pgxx.NewClient(pool))

	execID := "node-exec-" + id.NewJobID().String()
	jid := id.NewJobID()
	nid := id.NewNodeID()
	now := time.Now().UTC().Truncate(time.Millisecond)
	e := newNodeExec(execID, jid, nid, graph.StatusProcessing, &now, nil)

	require.NoError(t, r.Save(ctx, e))

	got, err := r.FindByJobNodeID(ctx, jid, nid.String())
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, execID, got.ID())
	assert.Equal(t, jid, got.JobID())
	assert.Equal(t, nid, got.NodeID())
	assert.Equal(t, graph.StatusProcessing, got.Status())
	require.NotNil(t, got.StartedAt())
	assert.Equal(t, now, got.StartedAt().UTC().Truncate(time.Millisecond))
	assert.Nil(t, got.CompletedAt())
}

func TestNodeExecution_FindByJobNodeID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeExecution(pgxx.NewClient(pool))

	got, err := r.FindByJobNodeID(ctx, id.NewJobID(), id.NewNodeID().String())
	assert.Nil(t, got)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestNodeExecution_Save_Upsert(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeExecution(pgxx.NewClient(pool))

	execID := "node-exec-upsert-" + id.NewJobID().String()
	jid := id.NewJobID()
	nid := id.NewNodeID()
	e := newNodeExec(execID, jid, nid, graph.StatusPending, nil, nil)
	require.NoError(t, r.Save(ctx, e))

	// Update to completed.
	now := time.Now().UTC().Truncate(time.Millisecond)
	e2 := newNodeExec(execID, jid, nid, graph.StatusCompleted, nil, &now)
	require.NoError(t, r.Save(ctx, e2))

	got, err := r.FindByJobNodeID(ctx, jid, nid.String())
	require.NoError(t, err)
	assert.Equal(t, graph.StatusCompleted, got.Status())
	require.NotNil(t, got.CompletedAt())
	assert.Equal(t, now, got.CompletedAt().UTC().Truncate(time.Millisecond))
}

func TestNodeExecution_Save_AllStatuses(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeExecution(pgxx.NewClient(pool))

	statuses := []graph.Status{
		graph.StatusPending,
		graph.StatusStarting,
		graph.StatusProcessing,
		graph.StatusCompleted,
		graph.StatusFailed,
	}
	jid := id.NewJobID()
	for _, s := range statuses {
		nid := id.NewNodeID()
		execID := "exec-" + string(s) + "-" + nid.String()
		e := newNodeExec(execID, jid, nid, s, nil, nil)
		require.NoError(t, r.Save(ctx, e), "status=%s", s)

		got, err := r.FindByJobNodeID(ctx, jid, nid.String())
		require.NoError(t, err, "status=%s", s)
		assert.Equal(t, s, got.Status(), "status=%s", s)
	}
}

func TestNodeExecution_Save_NilTimestamps(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewNodeExecution(pgxx.NewClient(pool))

	execID := "exec-nil-ts-" + id.NewJobID().String()
	jid := id.NewJobID()
	nid := id.NewNodeID()
	e := newNodeExec(execID, jid, nid, graph.StatusPending, nil, nil)
	require.NoError(t, r.Save(ctx, e))

	got, err := r.FindByJobNodeID(ctx, jid, nid.String())
	require.NoError(t, err)
	assert.Nil(t, got.StartedAt())
	assert.Nil(t, got.CompletedAt())
}
