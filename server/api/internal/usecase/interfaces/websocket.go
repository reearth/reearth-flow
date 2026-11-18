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
	// SaveVersion asks the websocket server to save the document as a version
	// labelled label, and saved is false when nothing changed since the last
	// one. The server reports each saved version to the API (POST
	// /auth/versions), which is what moves the project's updatedAt.
	SaveVersion(ctx context.Context, docID, label string) (saved bool, err error)

	Close() error
}

// ErrWebsocketRequestRejected is what a WebsocketClient call returns when the
// websocket server refuses the request itself (a 4xx), such as for a wrong API
// secret. Sending it again cannot succeed.
var ErrWebsocketRequestRejected = errors.New("websocket server rejected the request")

var (
	// ErrVersionSaveBusy reports that another save of the document held it for
	// the websocket server's whole wait. The changes stay recorded, so a save
	// in progress or the next one includes them.
	ErrVersionSaveBusy = errors.New("another save of the document is in progress")
	// ErrVersionsUnsupported reports that the websocket server's storage keeps
	// no versions, as in some local setups.
	ErrVersionsUnsupported = errors.New("websocket server does not keep versions")
)

// IsVersionSaveSkipped reports whether a SaveVersion error leaves nothing
// wrong for the caller: no version was saved, and none is missing.
func IsVersionSaveSkipped(err error) bool {
	return errors.Is(err, ErrVersionSaveBusy) || errors.Is(err, ErrVersionsUnsupported)
}
