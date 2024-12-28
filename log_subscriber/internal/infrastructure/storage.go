package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/log-subscriber/internal/infrastructure/gcs"
	"github.com/reearth/reearth-flow/log-subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/log-subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/log-subscriber/pkg/log"
)

type StorageImpl struct {
	redis *redis.RedisStorage
	gcs   *gcs.GCSStorage
}

func NewStorageImpl(r *redis.RedisStorage, g *gcs.GCSStorage) gateway.LogStorage {
	return &StorageImpl{
		redis: r,
		gcs:   g,
	}
}

func (s *StorageImpl) SaveToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	return s.redis.SaveLogToRedis(ctx, event)
}

func (s *StorageImpl) SaveToGCS(ctx context.Context, event *domainLog.LogEvent) error {
	return s.gcs.SaveLogToGCS(ctx, event)
}
