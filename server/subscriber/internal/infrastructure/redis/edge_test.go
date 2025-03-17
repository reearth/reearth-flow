package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

func TestRedisStorage_SaveEdgeEventToRedis(t *testing.T) {
	ctx := context.Background()
	mClient := new(mockRedisClient)
	rStorage := NewRedisStorage(mClient)

	event := &edge.PassThroughEvent{
		WorkflowID: "wf-123",
		JobID:      "job-123",
		Status:     edge.StatusCompleted,
		Timestamp:  time.Date(2025, 1, 11, 9, 12, 54, 487779000, time.UTC),
		UpdatedEdges: []edge.UpdatedEdge{
			{ID: "edge-1", Status: edge.StatusCompleted},
			{ID: "edge-2", Status: edge.StatusFailed},
		},
	}

	jobEventsKey := fmt.Sprintf("edgeEvents:%s", event.JobID)
	serializedBytes, _ := json.Marshal(event)
	serialized := string(serializedBytes)

	mClient.On("LPush", ctx, jobEventsKey, mock.MatchedBy(func(args []interface{}) bool {
		return len(args) == 1 && args[0] == serialized
	})).Return(nil)
	mClient.On("Expire", ctx, jobEventsKey, 12*time.Hour).Return(nil)

	for _, updatedEdge := range event.UpdatedEdges {
		edgeKey := fmt.Sprintf("edge:%s:%s", event.JobID, updatedEdge.ID)
		edgeData := map[string]interface{}{
			"id":         updatedEdge.ID,
			"status":     updatedEdge.Status,
			"timestamp":  event.Timestamp,
			"workflowId": event.WorkflowID,
			"jobId":      event.JobID,
		}
		edgeDataBytes, _ := json.Marshal(edgeData)
		mClient.On("Set", ctx, edgeKey, string(edgeDataBytes), 12*time.Hour).Return(nil)
	}

	err := rStorage.SaveEdgeEventToRedis(ctx, event)
	assert.NoError(t, err)
	mClient.AssertExpectations(t)
}
