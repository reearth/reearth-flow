package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

type StorageImpl struct {
	redis *redis.RedisStorage
}

func NewStorageImpl(r *redis.RedisStorage) gateway.LogStorage {
	return &StorageImpl{
		redis: r,
	}
}

func (s *StorageImpl) SaveToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	return s.redis.SaveLogToRedis(ctx, event)
}
