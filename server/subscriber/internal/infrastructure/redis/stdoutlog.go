package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/stdoutlog"
)

const (
	stdoutLogKeyPrefix = "stdout-log:"
	stdoutLogKeyExpiry = 12 * time.Hour
)

func (r *RedisStorage) Save(ctx context.Context, event *stdoutlog.Event) error {
	if event == nil {
		return fmt.Errorf("cannot save nil stdout log event to Redis")
	}

	key := fmt.Sprintf("%s%s:%s:%s",
		stdoutLogKeyPrefix,
		event.WorkflowID,
		event.JobID,
		event.Timestamp.Format(time.RFC3339Nano),
	)

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		log.Printf("ERROR: Failed to marshal stdout log event for JobID=%s: %v", event.JobID, err)
		return fmt.Errorf("failed to marshal stdout log event: %w", err)
	}

	if err := r.client.Set(ctx, key, string(serializedBytes), stdoutLogKeyExpiry).Err(); err != nil {
		log.Printf("ERROR: Failed to set stdout log event in Redis for key %s: %v", key, err)
		return fmt.Errorf("failed to set stdout log event in Redis: %w", err)
	}

	log.Printf("DEBUG: Successfully saved stdout log event to Redis with key %s", key)
	return nil
}
