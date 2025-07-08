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

func (i *Asset) FindByProject(ctx context.Context, pid id.ProjectID, keyword *string, sort *asset.SortType, p *interfaces.PaginationParam) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, nil, err
	}

	// Since assets are already filtered by project and contain workspace info,
	// we can let the repository handle workspace filtering internally
	assets, pageInfo, err := i.repos.Asset.FindByProject(ctx, pid, repo.AssetFilter{
		Sort:       sort,
		Keyword:    keyword,
		Pagination: p,
	})
	if err != nil {
		return nil, nil, err
	}

	// If no assets found, no need to check workspace permissions
	if len(assets) == 0 {
		return assets, pageInfo, nil
	}

	// Get workspace ID from the first asset (all assets in a project share the same workspace)
	workspaceID := assets[0].Workspace()
	
	// Check if user has access to this workspace
	return Run2(
		ctx, i.repos,
		Usecase().WithReadableWorkspaces(workspaceID),
		func(ctx context.Context) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
			// Permission check passed, return the already fetched assets
			return assets, pageInfo, nil
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

	// Get project to find workspace
	project, err := i.repos.Project.FindByID(ctx, inp.ProjectID)
	if err != nil {
		return nil, err
	}

	url, size, err := i.gateways.File.UploadAsset(ctx, inp.File)
	if err != nil {
		return nil, err
	}

	previewType := asset.DetectPreviewTypeFromFile(inp.File)

	builder := asset.New().
		NewID().
		Project(inp.ProjectID).
		Workspace(project.Workspace()).
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
