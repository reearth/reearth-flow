package gql

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
)

type MockNodeDiagnosticsUsecase struct {
	mock.Mock
}

func (m *MockNodeDiagnosticsUsecase) GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	args := m.Called(ctx, jobID, nodeID)
	rows, _ := args.Get(0).([]*diagnostic.Diagnostic)
	return rows, args.Error(1)
}

func (m *MockNodeDiagnosticsUsecase) GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	args := m.Called(ctx, jobID)
	rows, _ := args.Get(0).([]*diagnostic.Diagnostic)
	return rows, args.Error(1)
}

func (m *MockNodeDiagnosticsUsecase) GetFailedNodes(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	args := m.Called(ctx, jobID)
	rows, _ := args.Get(0).([]*diagnostic.Diagnostic)
	return rows, args.Error(1)
}

func (m *MockNodeDiagnosticsUsecase) GetDroppedEventCount(ctx context.Context, jobID id.JobID) (*uint64, error) {
	args := m.Called(ctx, jobID)
	count, _ := args.Get(0).(*uint64)
	return count, args.Error(1)
}

func newTestLoaderDiagnostic(t *testing.T, jobID id.JobID, code string) *diagnostic.Diagnostic {
	t.Helper()
	d, err := diagnostic.NewBuilder().
		JobID(jobID).
		Code(code).
		Category("internal").
		Severity("warn").
		Message("test diagnostic").
		Build()
	require.NoError(t, err)
	return d
}

func TestDiagnosticLoader_GetNodeDiagnostics(t *testing.T) {
	ctx := context.Background()
	jID := id.NewJobID()
	gqlJobID := gqlmodel.ID(jID.String())

	t.Run("success", func(t *testing.T) {
		mockUsecase := new(MockNodeDiagnosticsUsecase)
		loader := NewDiagnosticLoader(mockUsecase)
		rows := []*diagnostic.Diagnostic{newTestLoaderDiagnostic(t, jID, "internal.unclassified")}
		mockUsecase.On("GetNodeDiagnostics", ctx, jID, "node-1").Return(rows, nil)

		got, err := loader.GetNodeDiagnostics(ctx, gqlJobID, "node-1")
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "internal.unclassified", got[0].Code)
		mockUsecase.AssertExpectations(t)
	})

	t.Run("usecase error", func(t *testing.T) {
		mockUsecase := new(MockNodeDiagnosticsUsecase)
		loader := NewDiagnosticLoader(mockUsecase)
		mockUsecase.On("GetNodeDiagnostics", ctx, jID, "node-1").Return(nil, errors.New("usecase error"))

		got, err := loader.GetNodeDiagnostics(ctx, gqlJobID, "node-1")
		assert.Error(t, err)
		assert.Nil(t, got)
	})

	t.Run("invalid job id", func(t *testing.T) {
		mockUsecase := new(MockNodeDiagnosticsUsecase)
		loader := NewDiagnosticLoader(mockUsecase)

		got, err := loader.GetNodeDiagnostics(ctx, gqlmodel.ID("not-a-uuid"), "node-1")
		assert.Error(t, err)
		assert.Nil(t, got)
		mockUsecase.AssertNotCalled(t, "GetNodeDiagnostics")
	})
}

func TestDiagnosticLoader_GetFailedNodes(t *testing.T) {
	ctx := context.Background()
	jID := id.NewJobID()
	gqlJobID := gqlmodel.ID(jID.String())

	t.Run("success", func(t *testing.T) {
		mockUsecase := new(MockNodeDiagnosticsUsecase)
		loader := NewDiagnosticLoader(mockUsecase)
		rows := []*diagnostic.Diagnostic{newTestLoaderDiagnostic(t, jID, "internal.invariant_violation")}
		mockUsecase.On("GetFailedNodes", ctx, jID).Return(rows, nil)

		got, err := loader.GetFailedNodes(ctx, gqlJobID)
		assert.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, "internal.invariant_violation", got[0].Code)
		mockUsecase.AssertExpectations(t)
	})

	t.Run("usecase error", func(t *testing.T) {
		mockUsecase := new(MockNodeDiagnosticsUsecase)
		loader := NewDiagnosticLoader(mockUsecase)
		mockUsecase.On("GetFailedNodes", ctx, jID).Return(nil, errors.New("usecase error"))

		got, err := loader.GetFailedNodes(ctx, gqlJobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}

func TestDiagnosticLoader_GetDroppedEventCount(t *testing.T) {
	ctx := context.Background()
	jID := id.NewJobID()
	gqlJobID := gqlmodel.ID(jID.String())

	t.Run("success", func(t *testing.T) {
		mockUsecase := new(MockNodeDiagnosticsUsecase)
		loader := NewDiagnosticLoader(mockUsecase)
		dropped := uint64(3)
		mockUsecase.On("GetDroppedEventCount", ctx, jID).Return(&dropped, nil)

		got, err := loader.GetDroppedEventCount(ctx, gqlJobID)
		assert.NoError(t, err)
		require.NotNil(t, got)
		assert.Equal(t, 3, *got)
	})

	t.Run("nil count is not an error", func(t *testing.T) {
		mockUsecase := new(MockNodeDiagnosticsUsecase)
		loader := NewDiagnosticLoader(mockUsecase)
		mockUsecase.On("GetDroppedEventCount", ctx, jID).Return(nil, nil)

		got, err := loader.GetDroppedEventCount(ctx, gqlJobID)
		assert.NoError(t, err)
		assert.Nil(t, got)
	})

	t.Run("usecase error", func(t *testing.T) {
		mockUsecase := new(MockNodeDiagnosticsUsecase)
		loader := NewDiagnosticLoader(mockUsecase)
		mockUsecase.On("GetDroppedEventCount", ctx, jID).Return(nil, errors.New("usecase error"))

		got, err := loader.GetDroppedEventCount(ctx, gqlJobID)
		assert.Error(t, err)
		assert.Nil(t, got)
	})
}
