package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

func (r *RedisStorage) SaveDiagnosticToRedis(ctx context.Context, event *diagnostic.DiagnosticEvent) error {
	if event == nil {
		log.Printf("ERROR: Attempt to save nil diagnostic event to Redis")
		return fmt.Errorf("cannot save nil diagnostic event to Redis")
	}

	nodeSegment := "_job"
	if event.NodeID != nil && *event.NodeID != "" {
		nodeSegment = *event.NodeID
	}

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		log.Printf("ERROR: Failed to marshal diagnostic event for JobID=%s: %v", event.JobID, err)
		return fmt.Errorf("failed to marshal diagnostic event: %w", err)
	}
	serialized := string(serializedBytes)

	nodeKey := fmt.Sprintf("diagnostics:%s:%s", event.JobID, nodeSegment)
	if err := r.tracedLPush(ctx, nodeKey, serialized); err != nil {
		log.Printf("ERROR: Failed to push diagnostic event to Redis list %s: %v", nodeKey, err)
		return fmt.Errorf("failed to push diagnostic event to Redis list: %w", err)
	}
	if err := r.tracedExpire(ctx, nodeKey, 24*time.Hour); err != nil {
		log.Printf("WARNING: Failed to set expiration on Redis key %s: %v", nodeKey, err)
	}

	jobKey := fmt.Sprintf("diagnostics:%s", event.JobID)
	if err := r.tracedLPush(ctx, jobKey, serialized); err != nil {
		log.Printf("ERROR: Failed to push diagnostic event to Redis list %s: %v", jobKey, err)
		return fmt.Errorf("failed to push diagnostic event to Redis list: %w", err)
	}
	if err := r.tracedExpire(ctx, jobKey, 24*time.Hour); err != nil {
		log.Printf("WARNING: Failed to set expiration on Redis key %s: %v", jobKey, err)
	}

	return nil
}
