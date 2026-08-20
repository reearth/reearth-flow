package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/redis/go-redis/v9"

	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

// userFacingLogStreamRetention matches the legacy per-line key TTL so the stream
// keeps exactly the same window, trimmed by MINID rather than an invented count.
const userFacingLogStreamRetention = 12 * time.Hour

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
	if err := r.tracedSet(ctx, key, serialized, userFacingLogStreamRetention); err != nil {
		return fmt.Errorf("failed to save user facing log to redis: %w", err)
	}

	if err := r.saveUserFacingLogToStream(ctx, event.JobID, event.Timestamp, serialized); err != nil {
		return err
	}

	return nil
}

func (r *RedisStorage) saveUserFacingLogToStream(ctx context.Context, jobID string, ts time.Time, serialized string) error {
	streamKey := fmt.Sprintf("userfacinglog:%s", jobID)
	minID := fmt.Sprintf("%d", time.Now().Add(-userFacingLogStreamRetention).UnixMilli())

	if err := r.tracedXAdd(ctx, &redis.XAddArgs{
		Stream: streamKey,
		MinID:  minID,
		Approx: true,
		Values: map[string]interface{}{
			"data":        serialized,
			"timestampMs": ts.UnixMilli(),
		},
	}); err != nil {
		return fmt.Errorf("failed to add user facing log to redis stream: %w", err)
	}

	// MINID already trims stale entries; a missed TTL on an idle stream key is
	// cheap, so don't fail the whole write (and trigger redelivery) over it.
	if err := r.tracedExpire(ctx, streamKey, userFacingLogStreamRetention); err != nil {
		log.Printf("WARN: failed to set expiration on redis stream %s: %v", streamKey, err)
	}

	return nil
}
