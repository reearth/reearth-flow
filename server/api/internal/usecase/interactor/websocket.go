package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	ws "github.com/reearth/reearth-flow/api/pkg/websocket"
)

// TODO(2026-09): delete this file — tracked in reearth/reearth-flow#2334.
// Websocket is dead code — NewWebsocket has no
// callers, and interfaces.Container.Websocket is typed WebsocketClient, so the
// GraphQL resolvers talk to the client directly (note they call CopyDocument /
// ImportDocument, which do not even exist on this type; the CopyProject /
// ImportProject wrappers below are unreachable). Kept for one release in case
// something out-of-tree constructs it; remove once that is ruled out. Do not add
// new methods here — add them to interfaces.WebsocketClient instead.
type Websocket struct {
	client interfaces.WebsocketClient
}

func NewWebsocket(client interfaces.WebsocketClient) *Websocket {
	return &Websocket{client: client}
}

func (i *Websocket) GetLatest(ctx context.Context, id string) (*ws.Document, error) {
	return i.client.GetLatest(ctx, id)
}

func (i *Websocket) GetHistory(ctx context.Context, id string) ([]*ws.History, error) {
	return i.client.GetHistory(ctx, id)
}

func (i *Websocket) GetHistoryByVersion(ctx context.Context, id string, version int) (*ws.History, error) {
	return i.client.GetHistoryByVersion(ctx, id, version)
}

func (i *Websocket) GetHistoryMetadata(ctx context.Context, id string) ([]*ws.HistoryMetadata, error) {
	return i.client.GetHistoryMetadata(ctx, id)
}

func (i *Websocket) Rollback(ctx context.Context, id string, version int) (*ws.Document, error) {
	return i.client.Rollback(ctx, id, version)
}

func (i *Websocket) FlushToGCS(ctx context.Context, id string) error {
	return i.client.FlushToGCS(ctx, id)
}

func (i *Websocket) CreateSnapshot(ctx context.Context, docID string, version int, name string) (*ws.Document, error) {
	return i.client.CreateSnapshot(ctx, docID, version, name)
}

func (i *Websocket) CopyProject(ctx context.Context, id string, source string) error {
	return i.client.CopyDocument(ctx, id, source)
}

func (i *Websocket) ImportProject(ctx context.Context, id string, data []byte) error {
	return i.client.ImportDocument(ctx, id, data)
}

func (i *Websocket) DeleteDocument(ctx context.Context, id string) error {
	return i.client.DeleteDocument(ctx, id)
}

func (i *Websocket) Close() error {
	return i.client.Close()
}
