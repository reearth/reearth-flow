package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

func (r *RedisStorage) SaveToRedis(ctx context.Context, event *job.JobCompleteEvent) error {
	key := fmt.Sprintf("job_complete:%s", event.JobID)

	data, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal job complete event: %w", err)
	}

	if err := r.tracedSet(ctx, key, data, 24*time.Hour); err != nil {
		log.Printf("ERROR: Failed to save job complete event to Redis: %v", err)
		return fmt.Errorf("failed to save to Redis: %w", err)
	}

	log.Printf("DEBUG: Saved job complete event to Redis for jobID=%s, result=%s", event.JobID, event.Result)
	return nil
}
