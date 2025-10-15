package interactor

import (
	"context"
	"errors"
	"fmt"
	"net/url"
	"path"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/reearth/reearthx/rerror"
)

type Asset struct {
	repos             *repo.Container
	workspaceRepo     workspace.Repo
	gateways          *gateway.Container
	permissionChecker gateway.PermissionChecker
}

func NewAsset(r *repo.Container, g *gateway.Container, permissionChecker gateway.PermissionChecker, workspaceRepo workspace.Repo) interfaces.Asset {
	return &Asset{
		repos:             r,
		workspaceRepo:     workspaceRepo,
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

func (i *Asset) FindByWorkspace(ctx context.Context, wid id.WorkspaceID, keyword *string, sort *asset.SortType, p *interfaces.PaginationParam) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
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
	user := adapter.User(ctx)
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

func (i *Asset) CreateFromUpload(ctx context.Context, inp interfaces.CreateAssetFromUploadParam) (result *asset.Asset, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	if inp.File != nil {
		return i.Create(ctx, interfaces.CreateAssetParam{
			WorkspaceID: inp.WorkspaceID,
			File:        inp.File,
			Name:        inp.Name,
		})
	}

	if inp.Token == "" {
		return nil, interfaces.ErrFileNotIncluded
	}

	var file *file.File

	a, err := Run1(
		ctx, i.repos,
		Usecase().Transaction(),
		func(ctx context.Context) (*asset.Asset, error) {
			uuid := inp.Token
			u, err := i.repos.AssetUpload.FindByID(ctx, uuid)
			if err != nil {
				return nil, err
			}
			if u.Expired(time.Now()) {
				return nil, rerror.ErrInternalBy(fmt.Errorf("expired upload token: %s", uuid))
			}
			file, err = i.gateways.File.UploadedAsset(ctx, u)
			if err != nil {
				return nil, err
			}

			publicURL, err := i.gateways.File.GetPublicAssetURL(uuid, u.FileName())
			if err != nil {
				return nil, err
			}
			var assetURL string
			if publicURL != nil {
				assetURL = publicURL.String()
			}

			// Use custom name if provided, otherwise use filename
			name := path.Base(file.Path)
			if inp.Name != nil && *inp.Name != "" {
				name = *inp.Name
			}

			// Get user ID from context
			user := adapter.User(ctx)
			if user == nil {
				return nil, interfaces.ErrOperationDenied
			}

			ab := asset.New().
				NewID().
				Workspace(inp.WorkspaceID).
				CreatedByUser(user.ID()).
				FileName(path.Base(file.Path)).
				Name(name).
				Size(uint64(file.Size)).
				URL(assetURL).
				ContentType(file.ContentType).
				UUID(uuid)

			a, err := ab.Build()
			if err != nil {
				return nil, err
			}

			if err := i.repos.Asset.Save(ctx, a); err != nil {
				return nil, err
			}

			return a, nil
		})

	if err != nil {
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

func (i *Asset) CreateUpload(ctx context.Context, inp interfaces.CreateAssetUploadParam) (*interfaces.AssetUpload, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	user := adapter.User(ctx)
	if user == nil {
		return nil, interfaces.ErrOperationDenied
	}

	if inp.ContentEncoding == "gzip" {
		inp.Filename = strings.TrimSuffix(inp.Filename, ".gz")
	}

	var param *gateway.IssueUploadAssetParam
	if inp.Cursor == "" {
		if inp.Filename == "" {
			// TODO: Change to the appropriate error
			return nil, interfaces.ErrFileNotIncluded
		}

		const week = 7 * 24 * time.Hour
		expiresAt := time.Now().Add(1 * week)
		param = &gateway.IssueUploadAssetParam{
			UUID:            uuid.New().String(),
			Filename:        inp.Filename,
			ContentLength:   inp.ContentLength,
			ContentType:     inp.ContentType,
			ContentEncoding: inp.ContentEncoding,
			ExpiresAt:       expiresAt,
			Cursor:          "",
		}
	} else {
		wrapped, err := parseWrappedUploadCursor(inp.Cursor)
		if err != nil {
			return nil, fmt.Errorf("parse cursor(%s): %w", inp.Cursor, err)
		}
		au, err := i.repos.AssetUpload.FindByID(ctx, wrapped.UUID)
		if err != nil {
			return nil, fmt.Errorf("find asset upload(uuid=%s): %w", wrapped.UUID, err)
		}
		if inp.WorkspaceID.Compare(au.Workspace()) != 0 {
			return nil, fmt.Errorf("unmatched workspace id(in=%s,db=%s)", inp.WorkspaceID, au.Workspace())
		}
		param = &gateway.IssueUploadAssetParam{
			UUID:            wrapped.UUID,
			Filename:        au.FileName(),
			ContentLength:   au.ContentLength(),
			ContentEncoding: au.ContentEncoding(),
			ContentType:     au.ContentType(),
			ExpiresAt:       au.ExpiresAt(),
			Cursor:          wrapped.Cursor,
		}
	}

	ws, err := i.workspaceRepo.FindByID(ctx, inp.WorkspaceID)
	if err != nil {
		return nil, err
	}

	param.Workspace = ws.ID().String()
	uploadLink, err := i.gateways.File.IssueUploadAssetLink(ctx, *param)
	if errors.Is(err, gateway.ErrUnsupportedOperation) {
		return nil, rerror.ErrNotFound
	}
	if err != nil {
		return nil, err
	}

	if inp.Cursor == "" {
		u := asset.NewUpload().
			UUID(param.UUID).
			Workspace(ws.ID()).
			FileName(param.Filename).
			ExpiresAt(param.ExpiresAt).
			ContentLength(uploadLink.ContentLength).
			ContentType(uploadLink.ContentType).
			ContentEncoding(uploadLink.ContentEncoding).
			Build()
		if err := i.repos.AssetUpload.Save(ctx, u); err != nil {
			return nil, err
		}
	}

	return &interfaces.AssetUpload{
		URL:             uploadLink.URL,
		UUID:            param.UUID,
		ContentType:     uploadLink.ContentType,
		ContentLength:   uploadLink.ContentLength,
		ContentEncoding: uploadLink.ContentEncoding,
		Next:            wrapUploadCursor(param.UUID, uploadLink.Next),
	}, nil
}

type wrappedUploadCursor struct {
	UUID   string
	Cursor string
}

func (c wrappedUploadCursor) String() string {
	return c.UUID + "_" + c.Cursor
}

func parseWrappedUploadCursor(c string) (*wrappedUploadCursor, error) {
	uuid, cursor, found := strings.Cut(c, "_")
	if !found {
		return nil, fmt.Errorf("separator not found")
	}
	return &wrappedUploadCursor{
		UUID:   uuid,
		Cursor: cursor,
	}, nil
}

func wrapUploadCursor(uuid, cursor string) string {
	if cursor == "" {
		return ""
	}
	return wrappedUploadCursor{UUID: uuid, Cursor: cursor}.String()
}
