package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

type WorkerConfig interface {
	FindByWorkspace(ctx context.Context, workspace id.WorkspaceID) (*workerconfig.WorkerConfig, error)
	FindByWorkspaces(ctx context.Context, workspaces []id.WorkspaceID) ([]*workerconfig.WorkerConfig, error)
	Save(ctx context.Context, config *workerconfig.WorkerConfig) error
	Remove(ctx context.Context, workspace id.WorkspaceID) error
}
