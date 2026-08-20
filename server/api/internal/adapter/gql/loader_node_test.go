package gql

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

type MockNodeExecutionUsecase struct {
	mock.Mock
}

func (m *MockNodeExecutionUsecase) FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	args := m.Called(ctx, jobID, nodeID)
	ne, _ := args.Get(0).(*graph.NodeExecution)
	return ne, args.Error(1)
}

func (m *MockNodeExecutionUsecase) GetNodeExecutions(ctx context.Context, jobID id.JobID) ([]*graph.NodeExecution, error) {
	args := m.Called(ctx, jobID)
	nodes, _ := args.Get(0).([]*graph.NodeExecution)
	return nodes, args.Error(1)
}

func (m *MockNodeExecutionUsecase) GetNodeExecution(ctx context.Context, jobID id.JobID, nodeID string) (*graph.NodeExecution, error) {
	args := m.Called(ctx, jobID, nodeID)
	ne, _ := args.Get(0).(*graph.NodeExecution)
	return ne, args.Error(1)
}

func (m *MockNodeExecutionUsecase) SubscribeToNode(ctx context.Context, jobID id.JobID, nodeID string) (chan *graph.NodeExecution, error) {
	args := m.Called(ctx, jobID, nodeID)
	ch, _ := args.Get(0).(chan *graph.NodeExecution)
	return ch, args.Error(1)
}

func (m *MockNodeExecutionUsecase) UnsubscribeFromNode(jobID id.JobID, nodeID string, ch chan *graph.NodeExecution) {
	m.Called(jobID, nodeID, ch)
}

func TestNodeExLoader_FindByJobID_Success(t *testing.T) {
	mockUsecase := new(MockNodeExecutionUsecase)
	loader := NewNodeExLoader(mockUsecase)
	ctx := context.Background()
	jobID := id.NewJobID()
	nodeID := id.NewNodeID()

	want := []*graph.NodeExecution{
		graph.NewNodeExecution("n1", jobID, nodeID, graph.StatusCompleted),
	}
	mockUsecase.On("GetNodeExecutions", ctx, jobID).Return(want, nil)

	got, err := loader.FindByJobID(ctx, gqlmodel.ID(jobID.String()))
	assert.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, gqlmodel.ID(want[0].ID()), got[0].ID)
	mockUsecase.AssertExpectations(t)
}

func TestNodeExLoader_FindByJobID_UsecaseError(t *testing.T) {
	mockUsecase := new(MockNodeExecutionUsecase)
	loader := NewNodeExLoader(mockUsecase)
	ctx := context.Background()
	jobID := id.NewJobID()

	mockUsecase.On("GetNodeExecutions", ctx, jobID).Return([]*graph.NodeExecution(nil), errors.New("usecase error"))

	got, err := loader.FindByJobID(ctx, gqlmodel.ID(jobID.String()))
	assert.Error(t, err)
	assert.Nil(t, got)
	mockUsecase.AssertExpectations(t)
}

func TestNodeExLoader_FindByJobID_InvalidJobID(t *testing.T) {
	mockUsecase := new(MockNodeExecutionUsecase)
	loader := NewNodeExLoader(mockUsecase)
	ctx := context.Background()

	got, err := loader.FindByJobID(ctx, gqlmodel.ID("not-a-valid-id"))
	assert.Error(t, err)
	assert.Nil(t, got)
	mockUsecase.AssertNotCalled(t, "GetNodeExecutions")
}
