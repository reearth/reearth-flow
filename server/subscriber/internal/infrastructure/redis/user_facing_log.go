package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"

	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

// userFacingLogStreamMaxLen bounds each job's stream independently of the 12h TTL below.
const userFacingLogStreamMaxLen = 10000

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
	if err := r.tracedSet(ctx, key, serialized, 12*time.Hour); err != nil {
		return fmt.Errorf("failed to save user facing log to redis: %w", err)
	}

	if err := r.saveUserFacingLogToStream(ctx, event.JobID, event.Timestamp, serialized); err != nil {
		return err
	}

	return nil
}

func (r *RedisStorage) saveUserFacingLogToStream(ctx context.Context, jobID string, ts time.Time, serialized string) error {
	streamKey := fmt.Sprintf("userfacinglog:%s", jobID)
	id := fmt.Sprintf("%d-*", ts.UnixMilli())

	if err := r.tracedXAdd(ctx, &redis.XAddArgs{
		Stream: streamKey,
		ID:     id,
		MaxLen: userFacingLogStreamMaxLen,
		Approx: true,
		Values: map[string]interface{}{"data": serialized},
	}); err != nil {
		return fmt.Errorf("failed to add user facing log to redis stream: %w", err)
	}

	if err := r.tracedExpire(ctx, streamKey, 12*time.Hour); err != nil {
		return fmt.Errorf("failed to set expiration on redis stream: %w", err)
	}

	return nil
}
