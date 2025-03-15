package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/document"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Document interface {
	Close() error
	GetLatest(ctx context.Context, docID id.DocumentID) (*document.Document, error)
	GetHistory(ctx context.Context, docID id.DocumentID) ([]*document.History, error)
	Rollback(ctx context.Context, id id.ProjectID, version int) (*document.Document, error)
}
