package infrastructure

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"
	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
	"github.com/reearth/reearth-flow/subscriber/pkg/node"
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

// nodeExecutionStorage is satisfied by both the mongo and postgres adapters.
type nodeExecutionStorage interface {
	SaveNodeExecution(ctx context.Context, jobID string, nodeExecution *node.NodeExecution) error
}

type nodeStorageImpl struct {
	redis *redis.RedisStorage
	db    nodeExecutionStorage
}

func NewNodeStorageImpl(r *redis.RedisStorage, db nodeExecutionStorage) gateway.NodeStorage {
	return &nodeStorageImpl{
		redis: r,
		db:    db,
	}
}

func (s *nodeStorageImpl) SaveNodeExecution(ctx context.Context, jobID string, nodeExecution *node.NodeExecution) error {
	return s.db.SaveNodeExecution(ctx, jobID, nodeExecution)
}

func (s *nodeStorageImpl) SaveToRedis(ctx context.Context, event *node.NodeStatusEvent) error {
	return s.redis.SaveNodeEventToRedis(ctx, event)
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
