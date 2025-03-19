package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
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

type edgeStorageImpl struct {
	redis *redis.RedisStorage
	mongo *mongo.MongoStorage
}

func NewEdgeStorageImpl(r *redis.RedisStorage, m *mongo.MongoStorage) gateway.EdgeStorage {
	return &edgeStorageImpl{
		redis: r,
		mongo: m,
	}
}

func (s *edgeStorageImpl) ConstructIntermediateDataURL(jobID, edgeID string) string {
	return s.mongo.ConstructIntermediateDataURL(jobID, edgeID)
}

func (s *edgeStorageImpl) FindEdgeExecution(ctx context.Context, jobID string, edgeID string) (*edge.EdgeExecution, error) {
	return s.mongo.FindEdgeExecution(ctx, jobID, edgeID)
}

func (s *edgeStorageImpl) SaveToRedis(ctx context.Context, event *edge.PassThroughEvent) error {
	return s.redis.SaveEdgeEventToRedis(ctx, event)
}

func (s *edgeStorageImpl) UpdateEdgeStatusInMongo(ctx context.Context, jobID string, edge *edge.EdgeExecution) error {
	return s.mongo.UpdateEdgeStatusInMongo(ctx, jobID, edge)
}
