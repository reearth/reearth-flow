package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/nodestatus"
)

func (r *RedisStorage) SaveEdgePassEventToRedis(ctx context.Context, event *nodestatus.EdgePassThroughEvent) error {
	key := fmt.Sprintf("edgePassEvents:%s", event.JobID)

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}
	serialized := string(serializedBytes)

	if err := r.client.LPush(ctx, key, serialized).Err(); err != nil {
		return fmt.Errorf("failed to push to Redis: %w", err)
	}

	if err := r.client.Expire(ctx, key, 12*time.Hour).Err(); err != nil {
		return fmt.Errorf("failed to set expiration: %w", err)
	}

	return nil
}
