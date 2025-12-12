package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStorageImpl struct {
	redis *redis.RedisStorage
}

func NewJobStorageImpl(redis *redis.RedisStorage) gateway.JobStorage {
	return &JobStorageImpl{
		redis: redis,
	}
}

func (s *JobStorageImpl) SaveToRedis(ctx context.Context, event *job.JobCompleteEvent) error {
	return s.redis.SaveToRedis(ctx, event)
}
