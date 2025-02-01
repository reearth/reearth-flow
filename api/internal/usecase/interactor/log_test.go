package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/stretchr/testify/assert"
)

type mockLogGateway struct {
	logs []*log.Log
	err  error
}

func (m *mockLogGateway) GetLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*log.Log, error) {
	return m.logs, m.err
}

func TestNewLogInteractor(t *testing.T) {
	t.Run("successfully create LogInteractor", func(t *testing.T) {
		redisMock := &mockLogGateway{}
		li := NewLogInteractor(redisMock)
		assert.NotNil(t, li)
	})
}

func TestLogInteractor_GetLogs(t *testing.T) {
	nodeID := log.NodeID(id.NewNodeID())
	jobID := id.NewJobID()
	redisLogs := []*log.Log{
		log.NewLog(jobID, &nodeID, time.Now(), log.LevelInfo, "redis log 1"),
		log.NewLog(jobID, &nodeID, time.Now(), log.LevelInfo, "redis log 2"),
	}
	redisMock := &mockLogGateway{logs: redisLogs}

	t.Run("get Redis logs", func(t *testing.T) {
		li := NewLogInteractor(redisMock)

		since := time.Now().Add(-30 * time.Minute)
		out, err := li.GetLogs(context.Background(), since, id.NewJobID(), &usecase.Operator{})
		assert.NoError(t, err)
		assert.Equal(t, redisLogs, out)
	})

	t.Run("redis error", func(t *testing.T) {
		brokenRedis := &mockLogGateway{err: errors.New("redis error")}
		li := NewLogInteractor(brokenRedis)

		since := time.Now()
		out, err := li.GetLogs(context.Background(), since, id.NewJobID(), &usecase.Operator{})
		assert.Nil(t, out)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to get logs from Redis")
	})

	t.Run("redis gateway is nil", func(t *testing.T) {
		li := NewLogInteractor(nil)
		since := time.Now().Add(-30 * time.Minute)
		out, err := li.GetLogs(context.Background(), since, jobID, &usecase.Operator{})
		assert.Nil(t, out)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "logsGatewayRedis is nil")
	})
}
