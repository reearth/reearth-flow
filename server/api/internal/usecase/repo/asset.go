package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type AssetFilter struct {
	Sort       *asset.SortType
	Keyword    *string
	Pagination *interfaces.PaginationParam
}

type Asset interface {
	Filtered(WorkspaceFilter) Asset
	FindByWorkspace(context.Context, id.WorkspaceID, AssetFilter) ([]*asset.Asset, *interfaces.PageBasedInfo, error)
	FindByID(context.Context, id.AssetID) (*asset.Asset, error)
	FindByIDs(context.Context, id.AssetIDList) ([]*asset.Asset, error)
	TotalSizeByWorkspace(context.Context, id.WorkspaceID) (uint64, error)
	Save(context.Context, *asset.Asset) error
	Delete(context.Context, id.AssetID) error
}
