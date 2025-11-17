package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/util"
)

type WorkerConfigLoader struct {
	usecase interfaces.WorkerConfig
}

func NewWorkerConfigLoader(usecase interfaces.WorkerConfig) *WorkerConfigLoader {
	return &WorkerConfigLoader{
		usecase: usecase,
	}
}

func (c *WorkerConfigLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.WorkerConfig, []error) {
	wids, err := util.TryMap(ids, gqlmodel.ToID[id.Workspace])
	if err != nil {
		return nil, []error{err}
	}

	res, err := c.usecase.FindByWorkspaces(ctx, wids)
	if err != nil {
		return nil, []error{err}
	}

	configMap := make(map[id.WorkspaceID]*gqlmodel.WorkerConfig)
	for _, cfg := range res {
		if cfg != nil {
			configMap[cfg.Workspace()] = gqlmodel.ToWorkerConfig(cfg)
		}
	}

	configs := make([]*gqlmodel.WorkerConfig, len(wids))
	for i, wid := range wids {
		configs[i] = configMap[wid]
	}

	return configs, nil
}

type WorkerConfigDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.WorkerConfig, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.WorkerConfig, []error)
}

func (c *WorkerConfigLoader) DataLoader(ctx context.Context) WorkerConfigDataLoader {
	return gqldataloader.NewWorkerConfigLoader(gqldataloader.WorkerConfigLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.WorkerConfig, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *WorkerConfigLoader) OrdinaryDataLoader(ctx context.Context) WorkerConfigDataLoader {
	return &ordinaryWorkerConfigLoader{
		fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.WorkerConfig, []error) {
			return c.Fetch(ctx, keys)
		},
	}
}

type ordinaryWorkerConfigLoader struct {
	fetch func(keys []gqlmodel.ID) ([]*gqlmodel.WorkerConfig, []error)
}

func (l *ordinaryWorkerConfigLoader) Load(key gqlmodel.ID) (*gqlmodel.WorkerConfig, error) {
	res, errs := l.fetch([]gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryWorkerConfigLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.WorkerConfig, []error) {
	return l.fetch(keys)
}
