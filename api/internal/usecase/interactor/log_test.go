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

func (m *mockLogGateway) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID) ([]*log.Log, error) {
	return m.logs, m.err
}

func TestNewLogInteractor(t *testing.T) {
	t.Run("successfully create LogInteractor", func(t *testing.T) {
		redisMock := &mockLogGateway{}
		gcsMock := &mockLogGateway{}
		li, err := NewLogInteractor(redisMock, gcsMock, 10*time.Minute)
		assert.NoError(t, err)
		assert.NotNil(t, li)
	})

	t.Run("nil gateway logs should return error", func(t *testing.T) {
		redisMock := &mockLogGateway{}
		li, err := NewLogInteractor(nil, redisMock, 10*time.Minute)
		assert.Nil(t, li)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "log gateways are required")

		li, err = NewLogInteractor(redisMock, nil, 10*time.Minute)
		assert.Nil(t, li)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "log gateways are required")
	})

	t.Run("non-positive recentLogsThreshold should default to 60 minutes", func(t *testing.T) {
		redisMock := &mockLogGateway{}
		gcsMock := &mockLogGateway{}
		li, err := NewLogInteractor(redisMock, gcsMock, -1*time.Hour)
		assert.NoError(t, err)
		assert.NotNil(t, li)

		logi := li.(*LogInteractor)
		assert.Equal(t, 60*time.Minute, logi.recentLogsThreshold)
	})
}

func TestLogInteractor_GetLogs(t *testing.T) {
	nodeID := log.NodeID(id.NewNodeID())
	workflowID := id.NewWorkflowID()
	jobID := id.NewJobID()
	redisLogs := []*log.Log{
		log.NewLog(workflowID, jobID, &nodeID, time.Now(), log.LevelInfo, "redis log 1"),
		log.NewLog(workflowID, jobID, &nodeID, time.Now(), log.LevelInfo, "redis log 2"),
	}
	gcsLogs := []*log.Log{
		log.NewLog(workflowID, jobID, &nodeID, time.Now(), log.LevelInfo, "gcs log 1"),
		log.NewLog(workflowID, jobID, &nodeID, time.Now(), log.LevelInfo, "gcs log 2"),
	}

	redisMock := &mockLogGateway{logs: redisLogs}
	gcsMock := &mockLogGateway{logs: gcsLogs}

	t.Run("use Redis logs if within threshold", func(t *testing.T) {
		li, err := NewLogInteractor(redisMock, gcsMock, 1*time.Hour)
		assert.NoError(t, err)

		// since is 30 minutes ago, which is < threshold => Redis
		since := time.Now().Add(-30 * time.Minute)
		out, err := li.GetLogs(context.Background(), since, id.NewWorkflowID(), id.NewJobID(), &usecase.Operator{})
		assert.NoError(t, err)
		assert.Equal(t, redisLogs, out)
	})

	t.Run("use GCS logs if outside threshold", func(t *testing.T) {
		li, err := NewLogInteractor(redisMock, gcsMock, 1*time.Hour)
		assert.NoError(t, err)

		// since is 2 hours ago, which is > threshold => GCS
		since := time.Now().Add(-2 * time.Hour)
		out, err := li.GetLogs(context.Background(), since, id.NewWorkflowID(), id.NewJobID(), &usecase.Operator{})
		assert.NoError(t, err)
		assert.Equal(t, gcsLogs, out)
	})

	t.Run("redis error", func(t *testing.T) {
		brokenRedis := &mockLogGateway{err: errors.New("redis error")}
		li, err := NewLogInteractor(brokenRedis, gcsMock, 1*time.Hour)
		assert.NoError(t, err)

		since := time.Now() // within threshold => tries Redis
		out, err := li.GetLogs(context.Background(), since, id.NewWorkflowID(), id.NewJobID(), &usecase.Operator{})
		assert.Nil(t, out)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to get logs from Redis")
	})

	t.Run("gcs error", func(t *testing.T) {
		brokenGCS := &mockLogGateway{err: errors.New("gcs error")}
		li, err := NewLogInteractor(redisMock, brokenGCS, 1*time.Hour)
		assert.NoError(t, err)

		// since is older => tries GCS
		since := time.Now().Add(-2 * time.Hour)
		out, err := li.GetLogs(context.Background(), since, id.NewWorkflowID(), id.NewJobID(), &usecase.Operator{})
		assert.Nil(t, out)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to get logs from GCS")
	})
}
