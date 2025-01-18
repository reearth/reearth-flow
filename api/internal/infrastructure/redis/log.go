package redis

import (
	"context"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type redisLog struct {
	client *redis.Client
}

func NewRedisLog(client *redis.Client) gateway.Log {
	return &redisLog{client: client}
}

func (g *redisLog) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID) ([]*log.Log, error) {

	dummyLogs := []*log.Log{
		log.NewLog(id.NewWorkflowID(), id.NewJobID(), nil, log.LevelInfo, "Test log message 1 from redis"),
		log.NewLog(id.NewWorkflowID(), id.NewJobID(), nil, log.LevelDebug, "Test log message 2 from redis"),
	}

	return dummyLogs, nil
}
