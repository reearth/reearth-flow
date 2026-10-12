package cms

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/circuitbreaker"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

// mockCMS is a lightweight testify/mock implementation of gateway.CMS.
type mockCMS struct{ mock.Mock }

func (m *mockCMS) GetProject(ctx context.Context, idOrAlias string) (*cms.Project, error) {
	args := m.Called(ctx, idOrAlias)
	p, _ := args.Get(0).(*cms.Project)
	return p, args.Error(1)
}
func (m *mockCMS) ListProjects(ctx context.Context, in cms.ListProjectsInput) (*cms.ListProjectsOutput, error) {
	args := m.Called(ctx, in)
	o, _ := args.Get(0).(*cms.ListProjectsOutput)
	return o, args.Error(1)
}
func (m *mockCMS) GetAsset(ctx context.Context, in cms.GetAssetInput) (*cms.Asset, error) {
	args := m.Called(ctx, in)
	a, _ := args.Get(0).(*cms.Asset)
	return a, args.Error(1)
}
func (m *mockCMS) ListAssets(ctx context.Context, in cms.ListAssetsInput) (*cms.ListAssetsOutput, error) {
	args := m.Called(ctx, in)
	o, _ := args.Get(0).(*cms.ListAssetsOutput)
	return o, args.Error(1)
}
func (m *mockCMS) GetModel(ctx context.Context, in cms.GetModelInput) (*cms.Model, error) {
	args := m.Called(ctx, in)
	mo, _ := args.Get(0).(*cms.Model)
	return mo, args.Error(1)
}
func (m *mockCMS) ListModels(ctx context.Context, in cms.ListModelsInput) (*cms.ListModelsOutput, error) {
	args := m.Called(ctx, in)
	o, _ := args.Get(0).(*cms.ListModelsOutput)
	return o, args.Error(1)
}
func (m *mockCMS) ListItems(ctx context.Context, in cms.ListItemsInput) (*cms.ListItemsOutput, error) {
	args := m.Called(ctx, in)
	o, _ := args.Get(0).(*cms.ListItemsOutput)
	return o, args.Error(1)
}
func (m *mockCMS) GetModelExportURL(ctx context.Context, in cms.ModelExportInput) (*cms.ExportOutput, error) {
	args := m.Called(ctx, in)
	o, _ := args.Get(0).(*cms.ExportOutput)
	return o, args.Error(1)
}
func (m *mockCMS) GetModelGeoJSONExportURL(ctx context.Context, in cms.ExportInput) (*cms.ExportOutput, error) {
	args := m.Called(ctx, in)
	o, _ := args.Get(0).(*cms.ExportOutput)
	return o, args.Error(1)
}

// Compile-time check that the decorator satisfies the gateway interface.
var _ gateway.CMS = (*circuitBreakerCMS)(nil)

func testCfg() circuitbreaker.Config {
	cfg := circuitbreaker.DefaultConfig("cms-test")
	cfg.ConsecutiveFailures = 2
	cfg.Timeout = 20 * time.Millisecond
	cfg.Interval = time.Hour
	return cfg
}

func TestNewCircuitBreakerCMS_NilNext(t *testing.T) {
	assert.Nil(t, NewCircuitBreakerCMS(nil, circuitbreaker.DefaultConfig("cms")))
}

func TestCircuitBreakerCMS_PassesThroughOnSuccess(t *testing.T) {
	m := &mockCMS{}
	want := &cms.Project{ID: "p1"}
	m.On("GetProject", mock.Anything, "p1").Return(want, nil)

	c := NewCircuitBreakerCMS(m, testCfg())
	got, err := c.GetProject(context.Background(), "p1")
	assert.NoError(t, err)
	assert.Same(t, want, got)
	m.AssertExpectations(t)
}

func TestCircuitBreakerCMS_OpensAfterConsecutiveFailures(t *testing.T) {
	m := &mockCMS{}
	boom := errors.New("boom")
	m.On("GetProject", mock.Anything, "p1").Return(nil, boom)

	c := NewCircuitBreakerCMS(m, testCfg())

	// Trip threshold = 2.
	_, err := c.GetProject(context.Background(), "p1")
	assert.ErrorIs(t, err, boom)
	_, err = c.GetProject(context.Background(), "p1")
	assert.ErrorIs(t, err, boom)

	// The next call should short-circuit without touching the underlying CMS.
	_, err = c.GetProject(context.Background(), "p1")
	assert.ErrorIs(t, err, circuitbreaker.ErrOpen)

	// GetProject was invoked exactly twice on the underlying CMS.
	m.AssertNumberOfCalls(t, "GetProject", 2)
}

func TestCircuitBreakerCMS_PerReturnTypeBreakerIsolation(t *testing.T) {
	m := &mockCMS{}
	boom := errors.New("boom")
	// Trip the *Project breaker by failing GetProject.
	m.On("GetProject", mock.Anything, "p1").Return(nil, boom)
	// GetAsset stays healthy.
	m.On("GetAsset", mock.Anything, mock.Anything).Return(&cms.Asset{ID: "a1"}, nil)

	c := NewCircuitBreakerCMS(m, testCfg())

	_, _ = c.GetProject(context.Background(), "p1")
	_, _ = c.GetProject(context.Background(), "p1")
	_, err := c.GetProject(context.Background(), "p1")
	assert.ErrorIs(t, err, circuitbreaker.ErrOpen)

	// A different return-type method should still be reachable.
	got, err := c.GetAsset(context.Background(), cms.GetAssetInput{AssetID: "a1"})
	assert.NoError(t, err)
	assert.Equal(t, "a1", got.ID)
}

func TestCircuitBreakerCMS_ContextCancelDoesNotTrip(t *testing.T) {
	m := &mockCMS{}
	m.On("GetProject", mock.Anything, "p1").Return(nil, context.Canceled)

	c := NewCircuitBreakerCMS(m, testCfg())

	for i := 0; i < 5; i++ {
		_, err := c.GetProject(context.Background(), "p1")
		assert.ErrorIs(t, err, context.Canceled)
	}
	m.AssertNumberOfCalls(t, "GetProject", 5)
}
