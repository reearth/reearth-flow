package gql

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/util"
)

type TriggerLoader struct {
	usecase interfaces.Trigger
}

func NewTriggerLoader(usecase interfaces.Trigger) *TriggerLoader {
	return &TriggerLoader{usecase: usecase}
}

func (c *TriggerLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.Trigger, []error) {
	ids2, err := util.TryMap(ids, gqlmodel.ToID[id.Trigger])
	if err != nil {
		return nil, []error{err}
	}

	res, err := c.usecase.Fetch(ctx, ids2)
	if err != nil {
		return nil, []error{err}
	}

	triggers := make([]*gqlmodel.Trigger, 0, len(res))
	for _, trigger := range res {
		triggers = append(triggers, gqlmodel.ToTrigger(trigger))
	}

	return triggers, nil
}

func (c *TriggerLoader) FindByWorkspacePage(ctx context.Context, wsID gqlmodel.ID, keyword *string, pagination gqlmodel.PageBasedPagination) (*gqlmodel.TriggerConnection, error) {
	tid, err := gqlmodel.ToID[accountsid.Workspace](wsID)
	if err != nil {
		return nil, err
	}

	paginationParam := gqlmodel.ToPageBasedPagination(pagination)
	res, pi, err := c.usecase.FindByWorkspace(ctx, tid, paginationParam, keyword)
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.Trigger, 0, len(res))
	for _, t := range res {
		nodes = append(nodes, gqlmodel.ToTrigger(t))
	}

	return &gqlmodel.TriggerConnection{
		Nodes:      nodes,
		PageInfo:   gqlmodel.ToPageInfo(pi),
		TotalCount: int(pi.TotalCount),
	}, nil
}

// data loaders

type TriggerDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.Trigger, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.Trigger, []error)
}

func (c *TriggerLoader) DataLoader(ctx context.Context) TriggerDataLoader {
	return gqldataloader.NewTriggerLoader(gqldataloader.TriggerLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Trigger, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *TriggerLoader) OrdinaryDataLoader(ctx context.Context) TriggerDataLoader {
	return &ordinaryTriggerLoader{
		fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Trigger, []error) {
			return c.Fetch(ctx, keys)
		},
	}
}

type ordinaryTriggerLoader struct {
	fetch func(keys []gqlmodel.ID) ([]*gqlmodel.Trigger, []error)
}

func (l *ordinaryTriggerLoader) Load(key gqlmodel.ID) (*gqlmodel.Trigger, error) {
	res, errs := l.fetch([]gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryTriggerLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.Trigger, []error) {
	return l.fetch(keys)
}
