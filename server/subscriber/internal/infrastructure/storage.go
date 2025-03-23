package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
	"github.com/reearth/reearth-flow/subscriber/pkg/node"
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

type nodeStorageImpl struct {
	redis *redis.RedisStorage
	mongo *mongo.MongoStorage
}

func NewNodeStorageImpl(r *redis.RedisStorage, m *mongo.MongoStorage) gateway.NodeStorage {
	return &nodeStorageImpl{
		redis: r,
		mongo: m,
	}
}

func (s *nodeStorageImpl) FindNodeExecution(ctx context.Context, jobID string, nodeID string) (*node.NodeExecution, error) {
	return s.mongo.FindNodeExecution(ctx, jobID, nodeID)
}

func (s *nodeStorageImpl) SaveToMongo(ctx context.Context, jobID string, nodeExecution *node.NodeExecution) error {
	return s.mongo.SaveNodeExecutionToMongo(ctx, jobID, nodeExecution)
}

func (s *nodeStorageImpl) SaveToRedis(ctx context.Context, event *node.NodeStatusEvent) error {
	return s.redis.SaveNodeEventToRedis(ctx, event)
}
