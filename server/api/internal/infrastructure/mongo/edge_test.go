package mongo

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/mongox/mongotest"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestEdgeExecution_FindByJobEdgeID(t *testing.T) {
	c := mongotest.Connect(t)(t)
	ctx := context.Background()

	jobID1 := id.NewJobID()
	jobID2 := id.NewJobID()
	edgeID1 := "edge1"
	edgeID2 := "edge2"
	execID1 := "exec1"
	execID2 := "exec2"
	now := time.Now()

	_, _ = c.Collection("edgeExecutions").InsertMany(ctx, []any{
		bson.M{
			"id":        execID1,
			"edgeId":    edgeID1,
			"jobId":     jobID1.String(),
			"status":    string(edge.StatusInProgress),
			"startedAt": now,
		},
		bson.M{
			"id":        execID2,
			"edgeId":    edgeID2,
			"jobId":     jobID2.String(),
			"status":    string(edge.StatusCompleted),
			"startedAt": now,
			"endedAt":   now.Add(5 * time.Minute),
		},
	})

	r := NewEdgeExecution(mongox.NewClientWithDatabase(c))

	t.Run("find existing edge execution", func(t *testing.T) {
		got, err := r.FindByJobEdgeID(ctx, jobID1, edgeID1)

		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, execID1, got.ID())
		assert.Equal(t, edgeID1, got.EdgeID())
		assert.Equal(t, jobID1, got.JobID())
		assert.Equal(t, edge.StatusInProgress, got.Status())
	})

	t.Run("find another existing edge execution", func(t *testing.T) {
		got, err := r.FindByJobEdgeID(ctx, jobID2, edgeID2)

		assert.NoError(t, err)
		assert.NotNil(t, got)
		assert.Equal(t, execID2, got.ID())
		assert.Equal(t, edgeID2, got.EdgeID())
		assert.Equal(t, jobID2, got.JobID())
		assert.Equal(t, edge.StatusCompleted, got.Status())
	})

	t.Run("non-existent job ID", func(t *testing.T) {
		nonExistentJobID := id.NewJobID()
		got, err := r.FindByJobEdgeID(ctx, nonExistentJobID, edgeID1)

		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("non-existent edge ID", func(t *testing.T) {
		got, err := r.FindByJobEdgeID(ctx, jobID1, "non-existent-edge")

		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
