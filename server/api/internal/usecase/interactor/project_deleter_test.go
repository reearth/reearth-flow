package interactor

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	ws "github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
)

// failingRemoveByProjectJobRepo fails the last step of the deletion transaction,
// so Project.Delete's WithinTransaction call returns an error.
type failingRemoveByProjectJobRepo struct {
	*memory.Job
	err error
}

func (f *failingRemoveByProjectJobRepo) RemoveByProject(context.Context, id.ProjectID) error {
	return f.err
}

// retryableProjectRepo counts Remove calls without actually deleting, mirroring
// a serializable transaction: a retried attempt starts from the same committed
// row, since the failed attempt never took effect.
type retryableProjectRepo struct {
	repo.Project
	removeCalls int
}

func (r *retryableProjectRepo) Remove(ctx context.Context, pid id.ProjectID) error {
	r.removeCalls++
	return nil
}

// mockWebsocketClient implements interfaces.WebsocketClient for testing.
type mockWebsocketClient struct {
	deleteDocumentFunc func(ctx context.Context, docID string) error
	deletedDocIDs      []string
}

func (m *mockWebsocketClient) DeleteDocument(ctx context.Context, docID string) error {
	m.deletedDocIDs = append(m.deletedDocIDs, docID)
	if m.deleteDocumentFunc != nil {
		return m.deleteDocumentFunc(ctx, docID)
	}
	return nil
}

// Stub implementations for the rest of the interface.
func (m *mockWebsocketClient) GetLatest(context.Context, string) (*ws.Document, error) {
	return nil, nil
}
func (m *mockWebsocketClient) GetHistory(context.Context, string) ([]*ws.History, error) {
	return nil, nil
}
func (m *mockWebsocketClient) GetHistoryByVersion(context.Context, string, int) (*ws.History, error) {
	return nil, nil
}
func (m *mockWebsocketClient) GetHistoryMetadata(context.Context, string) ([]*ws.HistoryMetadata, error) {
	return nil, nil
}
func (m *mockWebsocketClient) GetNamedSnapshots(context.Context, string) ([]*ws.SnapshotMetadata, error) {
	return nil, nil
}
func (m *mockWebsocketClient) SaveNamedSnapshot(context.Context, string, string) (*ws.SnapshotMetadata, error) {
	return nil, nil
}
func (m *mockWebsocketClient) Rollback(context.Context, string, int) (*ws.Document, error) {
	return nil, nil
}
func (m *mockWebsocketClient) FlushToGCS(context.Context, string) error { return nil }
func (m *mockWebsocketClient) CreateSnapshot(context.Context, string, int, string) (*ws.Document, error) {
	return nil, nil
}
func (m *mockWebsocketClient) CopyDocument(context.Context, string, string) error { return nil }
func (m *mockWebsocketClient) ImportDocument(context.Context, string, []byte) error {
	return nil
}
func (m *mockWebsocketClient) Close() error { return nil }

var _ interfaces.WebsocketClient = (*mockWebsocketClient)(nil)

func TestProjectDeleter_Delete_RemovesProject(t *testing.T) {
	ctx := context.Background()
	projectRepo := memory.NewProject()

	prj := project.New().NewID().MustBuild()
	assert.NoError(t, projectRepo.Save(ctx, prj))

	deleter := ProjectDeleter{
		Project: projectRepo,
	}

	err := deleter.Delete(ctx, prj, true)
	assert.NoError(t, err)

	_, err = projectRepo.FindByID(ctx, prj.ID())
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestProjectDeleter_Delete_NilProject(t *testing.T) {
	deleter := ProjectDeleter{
		Project: memory.NewProject(),
	}

	// Should be a no-op
	err := deleter.Delete(context.Background(), nil, true)
	assert.NoError(t, err)
}

// A retried transaction must not destroy the collaborative document twice
// (and, more importantly, must not destroy it if the row it justifies never
// commits). ProjectDeleter no longer touches the websocket document at all;
// that happens in Project.Delete's post-commit step, asserted below.

func TestProject_Delete_DeletesWebsocketDocumentAfterCommit(t *testing.T) {
	ctx := context.Background()
	projectRepo := memory.NewProject()
	jobRepo := memory.NewJob()
	wsClient := &mockWebsocketClient{}

	prj := project.New().NewID().MustBuild()
	assert.NoError(t, projectRepo.Save(ctx, prj))

	uc := &Project{
		projectRepo:       projectRepo,
		jobRepo:           jobRepo,
		websocket:         wsClient,
		transaction:       usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
		permissionChecker: NewMockPermissionChecker(nil),
	}

	assert.NoError(t, uc.Delete(ctx, prj.ID()))
	assert.Equal(t, []string{prj.ID().String()}, wsClient.deletedDocIDs)
}

func TestProject_Delete_SerializationRetryDeletesDocumentOnce(t *testing.T) {
	ctx := context.Background()
	memProjectRepo := memory.NewProject()
	projectRepo := &retryableProjectRepo{Project: memProjectRepo}
	jobRepo := memory.NewJob()
	wsClient := &mockWebsocketClient{}
	tx := &retryingTransactor{}

	prj := project.New().NewID().MustBuild()
	assert.NoError(t, memProjectRepo.Save(ctx, prj))

	uc := &Project{
		projectRepo:       projectRepo,
		jobRepo:           jobRepo,
		websocket:         wsClient,
		transaction:       tx,
		permissionChecker: NewMockPermissionChecker(nil),
	}

	assert.NoError(t, uc.Delete(ctx, prj.ID()))

	// The closure really did run twice...
	assert.Equal(t, 2, tx.runs)
	assert.Equal(t, 2, projectRepo.removeCalls)

	// ...but the document was only destroyed once.
	assert.Equal(t, []string{prj.ID().String()}, wsClient.deletedDocIDs)
}

func TestProject_Delete_DoesNotDeleteDocumentWhenTransactionFails(t *testing.T) {
	ctx := context.Background()
	projectRepo := memory.NewProject()
	wsClient := &mockWebsocketClient{}

	prj := project.New().NewID().MustBuild()
	assert.NoError(t, projectRepo.Save(ctx, prj))

	uc := &Project{
		projectRepo:       projectRepo,
		jobRepo:           &failingRemoveByProjectJobRepo{Job: memory.NewJob(), err: errors.New("db unavailable")},
		websocket:         wsClient,
		transaction:       usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
		permissionChecker: NewMockPermissionChecker(nil),
	}

	err := uc.Delete(ctx, prj.ID())
	assert.Error(t, err)
	assert.Empty(t, wsClient.deletedDocIDs)
}
