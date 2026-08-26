package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

type logStorageImpl struct {
	redis *redis.RedisStorage
}

func NewLogStorageImpl(r *redis.RedisStorage) gateway.LogStorage {
	return &logStorageImpl{
		redis: r,
	}
}

func (s *logStorageImpl) SaveToRedis(ctx context.Context, event *domainLog.LogEvent) error {
	return s.redis.SaveLogToRedis(ctx, event)
}

type userFacingLogStorageImpl struct {
	redis *redis.RedisStorage
}

func NewUserFacingLogStorageImpl(r *redis.RedisStorage) gateway.UserFacingLogStorage {
	return &userFacingLogStorageImpl{
		redis: r,
	}
}

func (s *userFacingLogStorageImpl) SaveToRedis(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	return s.redis.SaveUserFacingLogToRedis(ctx, event)
}
