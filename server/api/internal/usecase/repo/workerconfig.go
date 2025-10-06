package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

type WorkerConfig interface {
	FindByWorkspace(context.Context, workerconfig.WorkspaceID) (*workerconfig.WorkerConfig, error)
	Save(context.Context, *workerconfig.WorkerConfig) error
	Delete(context.Context, workerconfig.ID) error
}
