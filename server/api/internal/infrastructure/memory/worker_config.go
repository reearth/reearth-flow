package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

type WorkerConfig struct {
	data map[id.WorkerConfigID]*workerconfig.WorkerConfig
	lock sync.RWMutex
}

func NewWorkerConfig() repo.WorkerConfig {
	return &WorkerConfig{
		data: map[id.WorkerConfigID]*workerconfig.WorkerConfig{},
	}
}

func (r *WorkerConfig) FindByID(_ context.Context, wid id.WorkerConfigID) (*workerconfig.WorkerConfig, error) {
	r.lock.RLock()
	defer r.lock.RUnlock()

	if cfg, ok := r.data[wid]; ok {
		return workerconfig.Clone(cfg), nil
	}
	return nil, nil
}

func (r *WorkerConfig) FindByIDs(_ context.Context, ids []id.WorkerConfigID) ([]*workerconfig.WorkerConfig, error) {
	r.lock.RLock()
	defer r.lock.RUnlock()

	result := make([]*workerconfig.WorkerConfig, 0, len(ids))
	for _, wid := range ids {
		if cfg, ok := r.data[wid]; ok {
			result = append(result, workerconfig.Clone(cfg))
		}
	}
	return result, nil
}

func (r *WorkerConfig) FindAll(_ context.Context) (*workerconfig.WorkerConfig, error) {
	r.lock.RLock()
	defer r.lock.RUnlock()

	for _, cfg := range r.data {
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

	r.data[config.ID()] = workerconfig.Clone(config)
	return nil
}

func (r *WorkerConfig) Remove(_ context.Context, wid id.WorkerConfigID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	delete(r.data, wid)
	return nil
}
