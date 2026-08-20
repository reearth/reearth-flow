package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"

	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

// logStreamMaxLen bounds each job's stream independently of the 12h TTL below.
const logStreamMaxLen = 10000

func (r *RedisStorage) SaveLogToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	const layoutWithMillis = "2006-01-02T15:04:05.000000Z"
	key := fmt.Sprintf("log:%s:%s:%s", event.WorkflowID, event.JobID, event.Timestamp.UTC().Format(layoutWithMillis))

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}
	serialized := string(serializedBytes)
	if err := r.tracedSet(ctx, key, serialized, 12*time.Hour); err != nil {
		return err
	}

	if err := r.saveLogToStream(ctx, event.JobID, event.Timestamp, serialized); err != nil {
		return err
	}

	return nil
}

func (r *RedisStorage) saveLogToStream(ctx context.Context, jobID string, ts time.Time, serialized string) error {
	streamKey := fmt.Sprintf("log:%s", jobID)
	id := fmt.Sprintf("%d-*", ts.UnixMilli())

	if err := r.tracedXAdd(ctx, &redis.XAddArgs{
		Stream: streamKey,
		ID:     id,
		MaxLen: logStreamMaxLen,
		Approx: true,
		Values: map[string]interface{}{"data": serialized},
	}); err != nil {
		return fmt.Errorf("failed to add log to redis stream: %w", err)
	}

	if err := r.tracedExpire(ctx, streamKey, 12*time.Hour); err != nil {
		return fmt.Errorf("failed to set expiration on redis stream: %w", err)
	}

	return nil
}
