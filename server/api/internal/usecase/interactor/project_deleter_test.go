package interactor

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	ws "github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/stretchr/testify/assert"
)

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

func TestProjectDeleter_Delete_CascadesToWebsocket(t *testing.T) {
	ctx := context.Background()
	projectRepo := memory.NewProject()

	prj := project.New().NewID().MustBuild()
	assert.NoError(t, projectRepo.Save(ctx, prj))

	wsClient := &mockWebsocketClient{}

	deleter := ProjectDeleter{
		Project:   projectRepo,
		Websocket: wsClient,
	}

	err := deleter.Delete(ctx, prj, true)
	assert.NoError(t, err)

	// Verify websocket DeleteDocument was called with the project ID
	assert.Equal(t, []string{prj.ID().String()}, wsClient.deletedDocIDs)
}

func TestProjectDeleter_Delete_ContinuesWhenWebsocketFails(t *testing.T) {
	ctx := context.Background()
	projectRepo := memory.NewProject()

	prj := project.New().NewID().MustBuild()
	assert.NoError(t, projectRepo.Save(ctx, prj))

	wsClient := &mockWebsocketClient{
		deleteDocumentFunc: func(ctx context.Context, docID string) error {
			return errors.New("websocket server unreachable")
		},
	}

	deleter := ProjectDeleter{
		Project:   projectRepo,
		Websocket: wsClient,
	}

	// Should succeed even though websocket deletion failed
	err := deleter.Delete(ctx, prj, true)
	assert.NoError(t, err)

	// Verify the attempt was made
	assert.Equal(t, []string{prj.ID().String()}, wsClient.deletedDocIDs)
}

func TestProjectDeleter_Delete_WorksWithNilWebsocket(t *testing.T) {
	ctx := context.Background()
	projectRepo := memory.NewProject()

	prj := project.New().NewID().MustBuild()
	assert.NoError(t, projectRepo.Save(ctx, prj))

	deleter := ProjectDeleter{
		Project:   projectRepo,
		Websocket: nil,
	}

	// Should succeed without websocket client
	err := deleter.Delete(ctx, prj, true)
	assert.NoError(t, err)
}

func TestProjectDeleter_Delete_NilProject(t *testing.T) {
	deleter := ProjectDeleter{
		Project:   memory.NewProject(),
		Websocket: &mockWebsocketClient{},
	}

	// Should be a no-op
	err := deleter.Delete(context.Background(), nil, true)
	assert.NoError(t, err)
}
