package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/util"
)

type AssetLoader struct {
	usecase interfaces.Asset
}

func NewAssetLoader(usecase interfaces.Asset) *AssetLoader {
	return &AssetLoader{usecase: usecase}
}

func (c *AssetLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.Asset, []error) {
	ids2, err := util.TryMap(ids, gqlmodel.ToID[id.Asset])
	if err != nil {
		return nil, []error{err}
	}

	res, err := c.usecase.Fetch(ctx, ids2)
	if err != nil {
		return nil, []error{err}
	}

	return util.Map(res, gqlmodel.ToAsset), nil
}

func (c *AssetLoader) FindByWorkspace(ctx context.Context, wID gqlmodel.ID, keyword *string, sort *asset.SortType, pagination *gqlmodel.PageBasedPagination) (*gqlmodel.AssetConnection, error) {
	wid, err := gqlmodel.ToID[accountdomain.Workspace](wID)
	if err != nil {
		return nil, err
	}

	assets, pi, err := c.usecase.FindByWorkspace(ctx, wid, keyword, sort, gqlmodel.ToPageBasedPagination(*pagination))
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.Asset, 0, len(assets))
	for _, a := range assets {
		nodes = append(nodes, gqlmodel.ToAsset(a))
	}

	return &gqlmodel.AssetConnection{
		Nodes:      nodes,
		PageInfo:   gqlmodel.ToPageInfo(pi),
		TotalCount: int(pi.TotalCount),
	}, nil
}

// data loader

type AssetDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.Asset, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.Asset, []error)
}

func (c *AssetLoader) DataLoader(ctx context.Context) AssetDataLoader {
	return gqldataloader.NewAssetLoader(gqldataloader.AssetLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Asset, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *AssetLoader) OrdinaryDataLoader(ctx context.Context) AssetDataLoader {
	return &ordinaryAssetLoader{ctx: ctx, c: c}
}

type ordinaryAssetLoader struct {
	ctx context.Context
	c   *AssetLoader
}

func (l *ordinaryAssetLoader) Load(key gqlmodel.ID) (*gqlmodel.Asset, error) {
	res, errs := l.c.Fetch(l.ctx, []gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryAssetLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.Asset, []error) {
	return l.c.Fetch(l.ctx, keys)
}
