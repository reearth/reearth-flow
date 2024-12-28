package redis

import (
	"context"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
	domainLog "github.com/reearth/reearth-flow/log-subscriber/pkg/log"
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

// Save LogEvents to Redis in JSON format
func (r *RedisStorage) SaveLogToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	key := fmt.Sprintf("log:%s:%s:%s", event.WorkflowID, event.JobID, event.Timestamp.UTC().Format(time.RFC3339))

	// Convert the event to a string and save it
	serialized := fmt.Sprintf(`{"workflowId":%q,"jobId":%q,"nodeId":%q,"logLevel":%q,"timestamp":%q,"message":%q}`,
		event.WorkflowID, event.JobID, valOrEmpty(event.NodeID),
		event.LogLevel, event.Timestamp.Format(time.RFC3339), event.Message)

	// Set TTL as 60 minutes
	if err := r.client.Set(ctx, key, serialized, 60*time.Minute).Err(); err != nil {
		return err
	}
	return nil
}

func valOrEmpty(strPtr *string) string {
	if strPtr == nil {
		return ""
	}
	return *strPtr
}