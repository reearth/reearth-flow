package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/websocket"
)

type WebsocketClient interface {
	GetLatest(ctx context.Context, docID string) (*websocket.Document, error)
	GetHistory(ctx context.Context, docID string) ([]*websocket.History, error)
	GetHistoryByVersion(ctx context.Context, docID string, version int) ([]*websocket.History, error)
	GetHistoryMetadata(ctx context.Context, docID string) ([]*websocket.HistoryMetadata, error)
	Rollback(ctx context.Context, id string, version int) (*websocket.Document, error)
	FlushToGCS(ctx context.Context, id string) error

	Close() error
}
