package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

func (r *RedisStorage) SaveEdgeEventToRedis(ctx context.Context, event *edge.PassThroughEvent) error {
	jobEventsKey := fmt.Sprintf("edgeEvents:%s", event.JobID)

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal edge event: %w", err)
	}

	serialized := string(serializedBytes)

	if err := r.client.LPush(ctx, jobEventsKey, serialized).Err(); err != nil {
		return fmt.Errorf("failed to push event to Redis list: %w", err)
	}

	r.client.Expire(ctx, jobEventsKey, 12*time.Hour)

	for _, updatedEdge := range event.UpdatedEdges {
		edgeKey := fmt.Sprintf("edge:%s:%s", event.JobID, updatedEdge.ID)

		edgeData := map[string]interface{}{
			"id":         updatedEdge.ID,
			"status":     updatedEdge.Status,
			"timestamp":  event.Timestamp,
			"workflowId": event.WorkflowID,
			"jobId":      event.JobID,
		}

		if updatedEdge.FeatureID != nil {
			edgeData["featureId"] = updatedEdge.FeatureID
		}

		edgeDataBytes, err := json.Marshal(edgeData)
		if err != nil {
			return fmt.Errorf("failed to marshal edge data: %w", err)
		}

		if err := r.client.Set(ctx, edgeKey, string(edgeDataBytes), 12*time.Hour).Err(); err != nil {
			return fmt.Errorf("failed to set edge status in Redis: %w", err)
		}
	}

	return nil
}
