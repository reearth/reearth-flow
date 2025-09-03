package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

func (r *RedisStorage) SaveUserFacingLogToRedis(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	const layoutWithMillis = "2006-01-02T15:04:05.000000Z"
	key := fmt.Sprintf("userfacinglog:%s:%s:%s",
		event.WorkflowID,
		event.JobID,
		event.Timestamp.UTC().Format(layoutWithMillis))

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal user facing log event: %w", err)
	}

	serialized := string(serializedBytes)
	if err := r.client.Set(ctx, key, serialized, 12*time.Hour).Err(); err != nil {
		return fmt.Errorf("failed to save user facing log to redis: %w", err)
	}

	return nil
}
