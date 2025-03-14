package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/mongo"
	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
	"github.com/reearth/reearth-flow/subscriber/pkg/nodestatus"
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

// NodeStatusStorageImpl implements gateway.NodeStatusStorage
type NodeStatusStorageImpl struct {
	redis *redis.RedisStorage
	mongo *mongo.MongoStorage
}

func NewNodeStatusStorageImpl(r *redis.RedisStorage, m *mongo.MongoStorage) gateway.NodeStatusStorage {
	return &NodeStatusStorageImpl{
		redis: r,
		mongo: m,
	}
}

func (s *NodeStatusStorageImpl) SaveEdgePassToRedis(ctx context.Context, event *nodestatus.EdgePassThroughEvent) error {
	return s.redis.SaveEdgePassEventToRedis(ctx, event)
}

func (s *NodeStatusStorageImpl) UpdateNodeExecutions(ctx context.Context, event *nodestatus.EdgePassThroughEvent) error {
	return s.mongo.UpdateNodeExecutions(ctx, event)
}
