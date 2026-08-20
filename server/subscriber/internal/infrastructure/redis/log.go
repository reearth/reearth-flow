package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"github.com/redis/go-redis/v9"

	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

// logStreamRetention matches the legacy per-line key TTL so the stream keeps
// exactly the same window, trimmed by MINID rather than an invented count.
const logStreamRetention = 12 * time.Hour

func (r *RedisStorage) SaveLogToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	const layoutWithMillis = "2006-01-02T15:04:05.000000Z"
	key := fmt.Sprintf("log:%s:%s:%s", event.WorkflowID, event.JobID, event.Timestamp.UTC().Format(layoutWithMillis))

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}
	serialized := string(serializedBytes)
	if err := r.tracedSet(ctx, key, serialized, logStreamRetention); err != nil {
		return err
	}

	if err := r.saveLogToStream(ctx, event.JobID, event.Timestamp, serialized); err != nil {
		return err
	}

	return nil
}

func (r *RedisStorage) saveLogToStream(ctx context.Context, jobID string, ts time.Time, serialized string) error {
	streamKey := fmt.Sprintf("log:%s", jobID)
	minID := fmt.Sprintf("%d", time.Now().Add(-logStreamRetention).UnixMilli())

	if err := r.tracedXAdd(ctx, &redis.XAddArgs{
		Stream: streamKey,
		MinID:  minID,
		Approx: true,
		Values: map[string]interface{}{
			"data":        serialized,
			"timestampMs": ts.UnixMilli(),
		},
	}); err != nil {
		return fmt.Errorf("failed to add log to redis stream: %w", err)
	}

	// MINID already trims stale entries; a missed TTL on an idle stream key is
	// cheap, so don't fail the whole write (and trigger redelivery) over it.
	if err := r.tracedExpire(ctx, streamKey, logStreamRetention); err != nil {
		log.Printf("WARN: failed to set expiration on redis stream %s: %v", streamKey, err)
	}

	return nil
}
