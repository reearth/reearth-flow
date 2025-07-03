package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/util"
)

type ParameterLoader struct {
	usecase interfaces.Parameter
}

func NewParameterLoader(usecase interfaces.Parameter) *ParameterLoader {
	return &ParameterLoader{usecase: usecase}
}

func (c *ParameterLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.Parameter, []error) {
	ids2, err := util.TryMap(ids, gqlmodel.ToID[id.Parameter])
	if err != nil {
		return nil, []error{err}
	}

	res, err := c.usecase.Fetch(ctx, ids2)
	if err != nil {
		return nil, []error{err}
	}

	parameters := make([]*gqlmodel.Parameter, 0, len(*res))
	for _, param := range *res {
		parameters = append(parameters, gqlmodel.ToParameter(param))
	}

	return parameters, nil
}

func (c *ParameterLoader) FindByProject(ctx context.Context, pID gqlmodel.ID) ([]*gqlmodel.Parameter, error) {
	tid, err := gqlmodel.ToID[id.Project](pID)
	if err != nil {
		return nil, err
	}
	res, err := c.usecase.FetchByProject(ctx, tid)
	if err != nil {
		return nil, err
	}

	var params []*gqlmodel.Parameter = nil
	if res != nil {
		params = make([]*gqlmodel.Parameter, 0, len(*res))
		for _, param := range *res {
			params = append(params, gqlmodel.ToParameter(param))
		}
	}
	return params, nil
}

// data loaders

type ParameterDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.Parameter, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.Parameter, []error)
}

func (c *ParameterLoader) DataLoader(ctx context.Context) ParameterDataLoader {
	return gqldataloader.NewParameterLoader(gqldataloader.ParameterLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Parameter, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *ParameterLoader) OrdinaryDataLoader(ctx context.Context) ParameterDataLoader {
	return &ordinaryParameterLoader{
		fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Parameter, []error) {
			return c.Fetch(ctx, keys)
		},
	}
}

type ordinaryParameterLoader struct {
	fetch func(keys []gqlmodel.ID) ([]*gqlmodel.Parameter, []error)
}

func (l *ordinaryParameterLoader) Load(key gqlmodel.ID) (*gqlmodel.Parameter, error) {
	res, errs := l.fetch([]gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryParameterLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.Parameter, []error) {
	return l.fetch(keys)
}
