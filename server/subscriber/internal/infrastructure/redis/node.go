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

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		log.Printf("ERROR: Failed to marshal node event for JobID=%s: %v", event.JobID, err)
		return fmt.Errorf("failed to marshal node event: %w", err)
	}
	log.Printf("DEBUG: Successfully serialized event, size=%d bytes", len(serializedBytes))

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

	// Flattened onto the individual node key (rather than nested, as the
	// wire event carries it) to match the flat shape `NodeEntry` on the api
	// side already uses for startedAt/completedAt/featureId.
	if event.Metrics != nil {
		nodeData["featuresProcessed"] = event.Metrics.FeaturesProcessed
		nodeData["featuresWritten"] = event.Metrics.FeaturesWritten
		nodeData["finishFeatureCount"] = event.Metrics.FinishFeatureCount
		log.Printf("DEBUG: Node %s has metrics=%+v", event.NodeID, *event.Metrics)
	}

	nodeDataBytes, err := json.Marshal(nodeData)
	if err != nil {
		log.Printf("ERROR: Failed to marshal node data for NodeID=%s: %v", event.NodeID, err)
		return fmt.Errorf("failed to marshal node data: %w", err)
	}

	if err := r.tracedSet(ctx, nodeKey, string(nodeDataBytes), 12*time.Hour); err != nil {
		log.Printf("ERROR: Failed to set node status in Redis for key %s: %v", nodeKey, err)
		return fmt.Errorf("failed to set node status in Redis: %w", err)
	}
	log.Printf("DEBUG: Successfully set node data in Redis with key %s and 12-hour expiration", nodeKey)

	if err := r.saveNodeStatusToHash(ctx, event.JobID, event.NodeID, string(nodeDataBytes)); err != nil {
		log.Printf("ERROR: Failed to set node status hash for JobID=%s, NodeID=%s: %v", event.JobID, event.NodeID, err)
		return err
	}

	log.Printf("DEBUG: Completed saving node data to Redis for JobID=%s, NodeID=%s", event.JobID, event.NodeID)
	return nil
}

func (r *RedisStorage) saveNodeStatusToHash(ctx context.Context, jobID, nodeID, serialized string) error {
	hashKey := fmt.Sprintf("node:%s", jobID)

	if err := r.tracedHSet(ctx, hashKey, nodeID, serialized); err != nil {
		return fmt.Errorf("failed to set node status in redis hash: %w", err)
	}

	if err := r.tracedExpire(ctx, hashKey, 12*time.Hour); err != nil {
		return fmt.Errorf("failed to set expiration on redis hash: %w", err)
	}

	return nil
}
