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
	FindByProject(context.Context, id.ProjectID, AssetFilter) ([]*asset.Asset, *interfaces.PageBasedInfo, error)
	FindByID(context.Context, id.AssetID) (*asset.Asset, error)
	FindByIDs(context.Context, id.AssetIDList) ([]*asset.Asset, error)
	TotalSizeByProject(context.Context, id.ProjectID) (uint64, error)
	Save(context.Context, *asset.Asset) error
	Remove(context.Context, id.AssetID) error
}
