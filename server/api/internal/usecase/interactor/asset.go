package interactor

import (
	"context"
	"net/url"
	"path"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type Asset struct {
	repos             *repo.Container
	gateways          *gateway.Container
	permissionChecker gateway.PermissionChecker
}

func NewAsset(r *repo.Container, g *gateway.Container, permissionChecker gateway.PermissionChecker) interfaces.Asset {
	return &Asset{
		repos:             r,
		gateways:          g,
		permissionChecker: permissionChecker,
	}
}

func (i *Asset) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceAsset, action)
}

func (i *Asset) Fetch(ctx context.Context, assets []id.AssetID) ([]*asset.Asset, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.repos.Asset.FindByIDs(ctx, assets)
}

func (i *Asset) FindByWorkspace(ctx context.Context, wid accountdomain.WorkspaceID, keyword *string, sort *asset.SortType, p *interfaces.PaginationParam) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, nil, err
	}

	return Run2(
		ctx, i.repos,
		Usecase().WithReadableWorkspaces(wid),
		func(ctx context.Context) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
			return i.repos.Asset.FindByWorkspace(ctx, wid, repo.AssetFilter{
				Sort:       sort,
				Keyword:    keyword,
				Pagination: p,
			})
		},
	)
}

func (i *Asset) Create(ctx context.Context, inp interfaces.CreateAssetParam) (result *asset.Asset, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	if inp.File == nil {
		return nil, interfaces.ErrFileNotIncluded
	}

	url, size, err := i.gateways.File.UploadAsset(ctx, inp.File)
	if err != nil {
		return nil, err
	}

	previewType := asset.DetectPreviewTypeFromFile(inp.File)

	builder := asset.New().
		NewID().
		Workspace(inp.WorkspaceID).
		CreatedByUser(inp.UserID).
		FileName(path.Base(inp.File.Path)).
		Name(path.Base(inp.File.Path)).
		Size(uint64(size)).
		URL(url.String()).
		ContentType(inp.File.ContentType).
		NewUUID().
		CoreSupport(true)
	
	if previewType != nil {
		builder = builder.Type(*previewType)
	}
	
	a, err := builder.Build()
	if err != nil {
		return nil, err
	}

	if err := i.repos.Asset.Save(ctx, a); err != nil {
		return nil, err
	}

	return a, nil
}

func (i *Asset) Remove(ctx context.Context, aid id.AssetID) (result id.AssetID, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return aid, err
	}

	return Run1(
		ctx, i.repos,
		Usecase().Transaction(),
		func(ctx context.Context) (id.AssetID, error) {
			asset, err := i.repos.Asset.FindByID(ctx, aid)
			if err != nil {
				return aid, err
			}

			if url, _ := url.Parse(asset.URL()); url != nil {
				if err := i.gateways.File.RemoveAsset(ctx, url); err != nil {
					return aid, err
				}
			}

			return aid, i.repos.Asset.Remove(ctx, aid)
		},
	)
}
