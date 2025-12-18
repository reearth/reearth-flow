package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

type WorkerConfig interface {
	FindByID(ctx context.Context, id id.WorkerConfigID) (*workerconfig.WorkerConfig, error)
	FindByIDs(ctx context.Context, ids []id.WorkerConfigID) ([]*workerconfig.WorkerConfig, error)
	FindAll(ctx context.Context) (*workerconfig.WorkerConfig, error)
	Save(ctx context.Context, config *workerconfig.WorkerConfig) error
	Remove(ctx context.Context, id id.WorkerConfigID) error
}
