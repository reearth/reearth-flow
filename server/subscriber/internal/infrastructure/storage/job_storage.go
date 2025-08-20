package storage

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStorageImpl struct {
	redis *redis.JobStorageRedis
	mongo *mongo.JobStorageMongo
}

func NewJobStorageImpl(redis *redis.JobStorageRedis, mongo *mongo.JobStorageMongo) *JobStorageImpl {
	return &JobStorageImpl{
		redis: redis,
		mongo: mongo,
	}
}

func (s *JobStorageImpl) SaveToRedis(ctx context.Context, event *job.JobStatusEvent) error {
	return s.redis.SaveToRedis(ctx, event)
}

func (s *JobStorageImpl) SaveToMongo(ctx context.Context, jobID string, jobRecord *job.Job) error {
	return s.mongo.SaveToMongo(ctx, jobID, jobRecord)
}
