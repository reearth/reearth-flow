package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

type RedisClient interface {
	Set(ctx context.Context, key string, value interface{}, expiration time.Duration) *redis.StatusCmd
}

type RedisStorage struct {
	client RedisClient
}

func NewRedisStorage(client RedisClient) *RedisStorage {
	return &RedisStorage{client: client}
}

func (r *RedisStorage) SaveLogToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	const layoutWithMillis = "2006-01-02T15:04:05.000000Z"
	key := fmt.Sprintf("log:%s:%s:%s", event.WorkflowID, event.JobID, event.Timestamp.UTC().Format(layoutWithMillis))

	serializedBytes, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}
	serialized := string(serializedBytes)
	if err := r.client.Set(ctx, key, serialized, 12*time.Hour).Err(); err != nil {
		return err
	}
	return nil
}
