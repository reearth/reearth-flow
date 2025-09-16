package interactor

import (
	"context"
	"net/url"
	"path"

	"github.com/reearth/reearth-flow/api/internal/adapter"
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

	// Use custom name if provided, otherwise use filename
	name := path.Base(inp.File.Path)
	if inp.Name != nil && *inp.Name != "" {
		name = *inp.Name
	}

	// Get user ID from context
	user := adapter.ReearthxUser(ctx)
	if user == nil {
		return nil, interfaces.ErrOperationDenied
	}

	builder := asset.New().
		NewID().
		Workspace(inp.WorkspaceID).
		CreatedByUser(user.ID()).
		FileName(path.Base(inp.File.Path)).
		Name(name).
		Size(uint64(size)).
		URL(url.String()).
		ContentType(inp.File.ContentType).
		NewUUID()

	a, err := builder.Build()
	if err != nil {
		return nil, err
	}

	if err := i.repos.Asset.Save(ctx, a); err != nil {
		return nil, err
	}

	return a, nil
}

func (i *Asset) Update(ctx context.Context, inp interfaces.UpdateAssetParam) (result *asset.Asset, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return Run1(
		ctx, i.repos,
		Usecase().Transaction(),
		func(ctx context.Context) (*asset.Asset, error) {
			oldAsset, err := i.repos.Asset.FindByID(ctx, inp.AssetID)
			if err != nil {
				return nil, err
			}

			// Check if name needs to be updated
			if inp.Name == nil || *inp.Name == "" || *inp.Name == oldAsset.Name() {
				// No changes needed, return existing asset
				return oldAsset, nil
			}

			// Since reearthx assets are immutable for name, we need to rebuild the asset
			// with the new name while preserving all other properties
			builder := asset.New().
				ID(oldAsset.ID()).
				Workspace(oldAsset.Workspace()).
				CreatedAt(oldAsset.CreatedAt()).
				FileName(oldAsset.FileName()).
				Name(*inp.Name). // Use the new name
				Size(oldAsset.Size()).
				URL(oldAsset.URL()).
				ContentType(oldAsset.ContentType()).
				UUID(oldAsset.UUID()).
				FlatFiles(oldAsset.FlatFiles()).
				Public(oldAsset.Public())

			// Set user or integration
			if oldAsset.User() != nil {
				builder = builder.CreatedByUser(*oldAsset.User())
			} else if oldAsset.Integration() != nil {
				builder = builder.CreatedByIntegration(oldAsset.Integration())
			}

			// Set thread if present
			if oldAsset.Thread() != nil {
				builder = builder.Thread(oldAsset.Thread())
			}

			// Set archive extraction status if present
			if oldAsset.ArchiveExtractionStatus() != nil {
				builder = builder.ArchiveExtractionStatus(*oldAsset.ArchiveExtractionStatus())
			}

			// Build the updated asset
			updatedAsset, err := builder.Build()
			if err != nil {
				return nil, err
			}

			// Save the updated asset
			if err := i.repos.Asset.Save(ctx, updatedAsset); err != nil {
				return nil, err
			}

			return updatedAsset, nil
		},
	)
}

func (i *Asset) Delete(ctx context.Context, aid id.AssetID) (result id.AssetID, err error) {
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
				if err := i.gateways.File.DeleteAsset(ctx, url); err != nil {
					return aid, err
				}
			}

			return aid, i.repos.Asset.Delete(ctx, aid)
		},
	)
}
