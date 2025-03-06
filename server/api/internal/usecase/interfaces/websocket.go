package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/websocket"
)

type WebsocketClient interface {
	GetLatest(ctx context.Context, docID string) (*websocket.Document, error)
	GetHistory(ctx context.Context, docID string) ([]*websocket.History, error)
	Rollback(ctx context.Context, id string, version int) (*websocket.Document, error)

	Close() error
}
