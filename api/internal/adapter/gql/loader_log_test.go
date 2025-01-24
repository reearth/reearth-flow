package gql

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

type MockLogUsecase struct {
	mock.Mock
}

func (m *MockLogUsecase) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID, operator *usecase.Operator) ([]*log.Log, error) {
	args := m.Called(ctx, since, workflowID, jobID, operator)
	return args.Get(0).([]*log.Log), args.Error(1)
}

func TestGetLogs_Success(t *testing.T) {
	mockUsecase := new(MockLogUsecase)
	loader := NewLogLoader(mockUsecase)
	ctx := context.Background()
	since := time.Now()
	workflowID := gqlmodel.ID(id.NewWorkflowID().String())
	jobID := gqlmodel.ID(id.NewJobID().String())

	mockLogs := []*log.Log{
		log.NewLog(id.NewWorkflowID(), id.NewJobID(), nil, time.Now().UTC(), log.LevelInfo, "Test log message 1 from gcs"),
		log.NewLog(id.NewWorkflowID(), id.NewJobID(), nil, time.Now().UTC(), log.LevelDebug, "Test log message 2 from gcs"),
	}

	mockUsecase.On("GetLogs", ctx, since, mock.Anything, mock.Anything, mock.Anything).Return(mockLogs, nil)

	logs, err := loader.GetLogs(ctx, since, workflowID, jobID)
	assert.NoError(t, err)
	assert.Len(t, logs, len(mockLogs))
	mockUsecase.AssertExpectations(t)
}

func TestGetLogs_UsecaseError(t *testing.T) {
	mockUsecase := new(MockLogUsecase)
	loader := NewLogLoader(mockUsecase)
	ctx := context.Background()
	since := time.Now()
	workflowID := gqlmodel.ID(id.NewWorkflowID().String())
	jobID := gqlmodel.ID(id.NewJobID().String())

	mockUsecase.On("GetLogs", ctx, since, mock.Anything, mock.Anything, mock.Anything).Return([]*log.Log(nil), errors.New("usecase error"))

	logs, err := loader.GetLogs(ctx, since, workflowID, jobID)
	assert.Error(t, err)
	assert.Nil(t, logs)
	mockUsecase.AssertExpectations(t)
}
