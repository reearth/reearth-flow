package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

func (r *RedisStorage) SaveNodeEventToRedis(ctx context.Context, event *node.NodeStatusEvent) error {
	if event == nil {
		log.Printf("ERROR: Attempt to save nil node event to Redis")
		return fmt.Errorf("cannot save nil node event to Redis")
	}

	log.Printf("DEBUG: Starting Redis save for NodeID=%s, JobID=%s, WorkflowID=%s with status %s",
		event.NodeID, event.JobID, event.WorkflowID, event.Status)

	jobNodesKey := fmt.Sprintf("nodeEvents:%s", event.JobID)
	log.Printf("DEBUG: Using Redis list key: %s", jobNodesKey)

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		log.Printf("ERROR: Failed to marshal node event for JobID=%s: %v", event.JobID, err)
		return fmt.Errorf("failed to marshal node event: %w", err)
	}
	log.Printf("DEBUG: Successfully serialized event, size=%d bytes", len(serializedBytes))

	serialized := string(serializedBytes)

	if err := r.client.LPush(ctx, jobNodesKey, serialized).Err(); err != nil {
		log.Printf("ERROR: Failed to push event to Redis list %s: %v", jobNodesKey, err)
		return fmt.Errorf("failed to push event to Redis list: %w", err)
	}
	log.Printf("DEBUG: Successfully pushed event to Redis list %s", jobNodesKey)

	if err := r.client.Expire(ctx, jobNodesKey, 12*time.Hour).Err(); err != nil {
		log.Printf("WARNING: Failed to set expiration on Redis key %s: %v", jobNodesKey, err)
	} else {
		log.Printf("DEBUG: Set 12-hour expiration on Redis key %s", jobNodesKey)
	}

	// Store individual node status
	nodeKey := fmt.Sprintf("node:%s:%s", event.JobID, event.NodeID)
	log.Printf("DEBUG: Setting individual node key: %s", nodeKey)

	nodeData := map[string]interface{}{
		"id":         event.NodeID,
		"jobId":      event.JobID,
		"nodeId":     event.NodeID,
		"status":     strings.ToUpper(string(event.Status)),
		"timestamp":  event.Timestamp,
		"workflowId": event.WorkflowID,
	}

	if event.FeatureID != nil {
		nodeData["featureId"] = *event.FeatureID
		log.Printf("DEBUG: Node %s has featureId=%s", event.NodeID, *event.FeatureID)
	}

	nodeDataBytes, err := json.Marshal(nodeData)
	if err != nil {
		log.Printf("ERROR: Failed to marshal node data for NodeID=%s: %v", event.NodeID, err)
		return fmt.Errorf("failed to marshal node data: %w", err)
	}

	if err := r.client.Set(ctx, nodeKey, string(nodeDataBytes), 12*time.Hour).Err(); err != nil {
		log.Printf("ERROR: Failed to set node status in Redis for key %s: %v", nodeKey, err)
		return fmt.Errorf("failed to set node status in Redis: %w", err)
	}
	log.Printf("DEBUG: Successfully set node data in Redis with key %s and 12-hour expiration", nodeKey)

	log.Printf("DEBUG: Completed saving node data to Redis for JobID=%s, NodeID=%s", event.JobID, event.NodeID)
	return nil
}
