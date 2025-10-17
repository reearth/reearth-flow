package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/asset"
)

type AssetUpload interface {
	Filtered(WorkspaceFilter) AssetUpload
	Save(ctx context.Context, upload *asset.Upload) error
	FindByID(ctx context.Context, uuid string) (*asset.Upload, error)
}
