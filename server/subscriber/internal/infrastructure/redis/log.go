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

// streamRetention is the window the legacy per-line keys already keep. The
// streams are trimmed by MINID against the same window so both paths expire
// together during the dual-write period.
const streamRetention = 12 * time.Hour

func (r *RedisStorage) SaveLogToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	const layoutWithMillis = "2006-01-02T15:04:05.000000Z"
	key := fmt.Sprintf("log:%s:%s:%s", event.WorkflowID, event.JobID, event.Timestamp.UTC().Format(layoutWithMillis))

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}
	serialized := string(serializedBytes)
	if err := r.tracedSet(ctx, key, serialized, streamRetention); err != nil {
		return err
	}

	if err := r.saveLogToStream(ctx, event.JobID, event.Timestamp, serialized); err != nil {
		return err
	}

	return nil
}

func (r *RedisStorage) saveLogToStream(ctx context.Context, jobID string, ts time.Time, serialized string) error {
	streamKey := fmt.Sprintf("log:%s", jobID)
	minID := fmt.Sprintf("%d", time.Now().Add(-streamRetention).UnixMilli())

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
	if err := r.tracedExpire(ctx, streamKey, streamRetention); err != nil {
		log.Printf("WARN: failed to set expiration on redis stream %s: %v", streamKey, err)
	}

	return nil
}
