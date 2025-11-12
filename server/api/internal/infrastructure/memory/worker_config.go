package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type WorkerConfig struct {
	data map[id.WorkspaceID]*workerconfig.WorkerConfig
	lock sync.RWMutex
}

func NewWorkerConfig() repo.WorkerConfig {
	return &WorkerConfig{
		data: map[id.WorkspaceID]*workerconfig.WorkerConfig{},
	}
}

func (r *WorkerConfig) FindByWorkspace(_ context.Context, workspace id.WorkspaceID) (*workerconfig.WorkerConfig, error) {
	r.lock.RLock()
	defer r.lock.RUnlock()

	if cfg, ok := r.data[workspace]; ok {
		return workerconfig.Clone(cfg), nil
	}
	return nil, nil
}

func (r *WorkerConfig) Save(_ context.Context, config *workerconfig.WorkerConfig) error {
	if config == nil {
		return nil
	}

	r.lock.Lock()
	defer r.lock.Unlock()

	r.data[config.Workspace()] = workerconfig.Clone(config)
	return nil
}

func (r *WorkerConfig) Remove(_ context.Context, workspace id.WorkspaceID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	delete(r.data, workspace)
	return nil
}
