package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type WorkerConfig interface {
	FindByWorkspace(ctx context.Context, workspace id.WorkspaceID) (*batchconfig.WorkerConfig, error)
	Save(ctx context.Context, config *batchconfig.WorkerConfig) error
	Remove(ctx context.Context, workspace id.WorkspaceID) error
}
