package interfaces

import (
	"context"
	"errors"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/websocket"
)

type WebsocketClient interface {
	GetLatest(ctx context.Context, docID string) (*websocket.Document, error)
	GetHistory(ctx context.Context, docID string) ([]*websocket.History, error)
	GetHistoryByVersion(ctx context.Context, docID string, version int) (*websocket.History, error)
	GetHistoryMetadata(ctx context.Context, docID string) ([]*websocket.HistoryMetadata, error)
	GetNamedSnapshots(ctx context.Context, docID string) ([]*websocket.SnapshotMetadata, error)
	SaveNamedSnapshot(ctx context.Context, docID, label string) (*websocket.SnapshotMetadata, error)
	GetSnapshotState(ctx context.Context, docID string, snapshotNumber int) (*websocket.SnapshotState, error)
	Rollback(ctx context.Context, id string, version int) (*websocket.Document, error)
	FlushToGCS(ctx context.Context, id string) error
	CreateSnapshot(ctx context.Context, docID string, version int, name string) (*websocket.Document, error)
	CopyDocument(ctx context.Context, docID string, source string) error
	ImportDocument(ctx context.Context, docID string, data []byte) error
	DeleteDocument(ctx context.Context, docID string) error
	// SetProjectLock tells the websocket server the project was locked or
	// unlocked at changedAt, so peers already editing it lose or regain write
	// access. changedAt orders a project's lock changes, so it must increase
	// with each change: the project's updatedAt after SetIsLocked does.
	SetProjectLock(ctx context.Context, docID string, locked bool, changedAt time.Time) error

	Close() error
}

// ErrWebsocketRequestRejected is what a WebsocketClient call returns when the
// websocket server refuses the request itself (a 4xx), such as for a wrong API
// secret. Sending it again cannot succeed.
var ErrWebsocketRequestRejected = errors.New("websocket server rejected the request")
