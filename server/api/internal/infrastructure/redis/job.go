package redis

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/log"
)

func (r *redisLog) GetJobCompleteEvent(ctx context.Context, jobID id.JobID) (*gateway.JobCompleteEvent, error) {
	key := fmt.Sprintf("job_complete:%s", jobID.String())

	data, err := r.client.Get(ctx, key).Result()
	if err != nil {
		// Key doesn't exist - worker hasn't reported yet
		if errors.Is(err, redis.Nil) {
			return nil, nil
		}
		log.Errorfc(ctx, "Failed to get job complete event from Redis: %v", err)
		return nil, fmt.Errorf("failed to get from Redis: %w", err)
	}

	var event gateway.JobCompleteEvent
	if err := json.Unmarshal([]byte(data), &event); err != nil {
		log.Errorfc(ctx, "Failed to unmarshal job complete event: %v", err)
		return nil, fmt.Errorf("failed to unmarshal: %w", err)
	}

	if err := r.client.Del(ctx, key).Err(); err != nil {
		log.Warnf("Failed to delete job complete event key: %v", err)
	}

	log.Debugfc(ctx, "Retrieved job complete event from Redis for jobID=%s, result=%s", jobID.String(), event.Result)
	return &event, nil
}
