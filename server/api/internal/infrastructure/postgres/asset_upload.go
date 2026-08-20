package postgres

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type AssetUpload struct {
	c *pgxx.Client
	f repo.WorkspaceFilter
}

var _ repo.AssetUpload = (*AssetUpload)(nil)

func NewAssetUpload(c *pgxx.Client) *AssetUpload {
	return &AssetUpload{c: c}
}

func (r *AssetUpload) Filtered(f repo.WorkspaceFilter) repo.AssetUpload {
	return &AssetUpload{c: r.c, f: r.f.Merge(f)}
}

func (r *AssetUpload) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *AssetUpload) FindByID(ctx context.Context, uuid string) (*asset.Upload, error) {
	row, err := r.q(ctx).GetAssetUpload(ctx, uuid)
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	return assetUploadFromRow(row)
}

func (r *AssetUpload) Save(ctx context.Context, upload *asset.Upload) error {
	if !r.f.CanWrite(upload.Workspace()) {
		return repo.ErrOperationDenied
	}
	if err := r.q(ctx).UpsertAssetUpload(ctx, assetUploadToParams(upload)); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func assetUploadToParams(u *asset.Upload) gen.UpsertAssetUploadParams {
	return gen.UpsertAssetUploadParams{
		Uuid:            u.UUID(),
		WorkspaceID:     u.Workspace().String(),
		FileName:        u.FileName(),
		ContentType:     u.ContentType(),
		ContentEncoding: u.ContentEncoding(),
		ContentLength:   u.ContentLength(),
		ExpiresAt:       u.ExpiresAt(),
	}
}

func assetUploadFromRow(row gen.AssetUpload) (*asset.Upload, error) {
	wid, err := accountsid.WorkspaceIDFrom(row.WorkspaceID)
	if err != nil {
		return nil, err
	}
	return asset.NewUpload().
		UUID(row.Uuid).
		Workspace(wid).
		FileName(row.FileName).
		ContentType(row.ContentType).
		ContentEncoding(row.ContentEncoding).
		ContentLength(row.ContentLength).
		ExpiresAt(row.ExpiresAt).
		Build(), nil
}
