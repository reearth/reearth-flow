package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

func (r *RedisStorage) SaveEdgeEventToRedis(ctx context.Context, event *edge.PassThroughEvent) error {
	if event == nil {
		log.Printf("ERROR: Attempt to save nil event to Redis")
		return fmt.Errorf("cannot save nil event to Redis")
	}

	log.Printf("DEBUG: Starting Redis save for JobID=%s, WorkflowID=%s with %d edges",
		event.JobID, event.WorkflowID, len(event.UpdatedEdges))

	jobEventsKey := fmt.Sprintf("edgeEvents:%s", event.JobID)
	log.Printf("DEBUG: Using Redis list key: %s", jobEventsKey)

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		log.Printf("ERROR: Failed to marshal edge event for JobID=%s: %v", event.JobID, err)
		return fmt.Errorf("failed to marshal edge event: %w", err)
	}
	log.Printf("DEBUG: Successfully serialized event, size=%d bytes", len(serializedBytes))

	serialized := string(serializedBytes)

	if err := r.client.LPush(ctx, jobEventsKey, serialized).Err(); err != nil {
		log.Printf("ERROR: Failed to push event to Redis list %s: %v", jobEventsKey, err)
		return fmt.Errorf("failed to push event to Redis list: %w", err)
	}
	log.Printf("DEBUG: Successfully pushed event to Redis list %s", jobEventsKey)

	// Set expiration on the list
	if err := r.client.Expire(ctx, jobEventsKey, 12*time.Hour).Err(); err != nil {
		log.Printf("WARNING: Failed to set expiration on Redis key %s: %v", jobEventsKey, err)
	} else {
		log.Printf("DEBUG: Set 12-hour expiration on Redis key %s", jobEventsKey)
	}

	// Process individual edge updates
	for i, updatedEdge := range event.UpdatedEdges {
		edgeKey := fmt.Sprintf("edge:%s:%s", event.JobID, updatedEdge.ID)
		log.Printf("DEBUG: Processing edge %d/%d, key=%s, status=%s",
			i+1, len(event.UpdatedEdges), edgeKey, updatedEdge.Status)

		edgeData := map[string]interface{}{
			"id":         updatedEdge.ID,
			"status":     updatedEdge.Status,
			"timestamp":  event.Timestamp,
			"workflowId": event.WorkflowID,
			"jobId":      event.JobID,
		}

		if updatedEdge.FeatureID != nil {
			edgeData["featureId"] = *updatedEdge.FeatureID
			log.Printf("DEBUG: Edge %s has featureId=%s", updatedEdge.ID, *updatedEdge.FeatureID)
		} else {
			log.Printf("DEBUG: Edge %s has no featureId", updatedEdge.ID)
		}

		edgeDataBytes, err := json.Marshal(edgeData)
		if err != nil {
			log.Printf("ERROR: Failed to marshal edge data for EdgeID=%s: %v", updatedEdge.ID, err)
			return fmt.Errorf("failed to marshal edge data: %w", err)
		}
		log.Printf("DEBUG: Successfully serialized edge data, size=%d bytes", len(edgeDataBytes))

		if err := r.client.Set(ctx, edgeKey, string(edgeDataBytes), 12*time.Hour).Err(); err != nil {
			log.Printf("ERROR: Failed to set edge status in Redis for key %s: %v", edgeKey, err)
			return fmt.Errorf("failed to set edge status in Redis: %w", err)
		}
		log.Printf("DEBUG: Successfully set edge data in Redis with key %s and 12-hour expiration", edgeKey)
	}

	log.Printf("DEBUG: Completed saving all edge data to Redis for JobID=%s", event.JobID)
	return nil
}
