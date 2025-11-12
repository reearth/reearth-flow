package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type WorkerConfig interface {
	FindByWorkspace(ctx context.Context, workspace id.WorkspaceID) (*workerconfig.WorkerConfig, error)
	Save(ctx context.Context, config *workerconfig.WorkerConfig) error
	Remove(ctx context.Context, workspace id.WorkspaceID) error
}
