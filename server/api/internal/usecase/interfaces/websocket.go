package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/websocket"
)

type WebsocketClient interface {
	GetLatest(ctx context.Context, docID string) (*websocket.Document, error)
	GetHistory(ctx context.Context, docID string) ([]*websocket.History, error)
	GetHistoryByVersion(ctx context.Context, docID string, version int) (*websocket.History, error)
	GetHistoryMetadata(ctx context.Context, docID string) ([]*websocket.HistoryMetadata, error)
	Rollback(ctx context.Context, id string, version int) (*websocket.Document, error)
	FlushToGCS(ctx context.Context, id string) error
	CreateSnapshot(ctx context.Context, docID string, version int, name string) (*websocket.Document, error)
	CopyDocument(ctx context.Context, docID string) error
	ImportDocument(ctx context.Context, docID string, data []byte) error

	Close() error
}
