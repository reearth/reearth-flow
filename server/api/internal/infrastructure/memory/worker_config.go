package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/batchconfig"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type WorkerConfig struct {
	lock sync.RWMutex
	data map[id.WorkspaceID]*batchconfig.WorkerConfig
}

func NewWorkerConfig() repo.WorkerConfig {
	return &WorkerConfig{
		data: map[id.WorkspaceID]*batchconfig.WorkerConfig{},
	}
}

func (r *WorkerConfig) FindByWorkspace(_ context.Context, workspace id.WorkspaceID) (*batchconfig.WorkerConfig, error) {
	r.lock.RLock()
	defer r.lock.RUnlock()

	if cfg, ok := r.data[workspace]; ok {
		return batchconfig.Clone(cfg), nil
	}
	return nil, nil
}

func (r *WorkerConfig) Save(_ context.Context, config *batchconfig.WorkerConfig) error {
	if config == nil {
		return nil
	}

	r.lock.Lock()
	defer r.lock.Unlock()

	r.data[config.Workspace()] = batchconfig.Clone(config)
	return nil
}

func (r *WorkerConfig) Remove(_ context.Context, workspace id.WorkspaceID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	delete(r.data, workspace)
	return nil
}
