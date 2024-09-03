package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
)

type AssetFilterType string

const (
	AssetFilterDate AssetFilterType = "DATE"
	AssetFilterSize AssetFilterType = "SIZE"
	AssetFilterName AssetFilterType = "NAME"
)

type CreateAssetParam struct {
	WorkspaceID accountdomain.WorkspaceID
	File        *file.File
}

var ErrCreateAssetFailed error = errors.New("failed to create asset")

type Asset interface {
	Fetch(context.Context, []id.AssetID, *usecase.Operator) ([]*asset.Asset, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *string, *asset.SortType, *usecasex.Pagination, *usecase.Operator) ([]*asset.Asset, *usecasex.PageInfo, error)
	Create(context.Context, CreateAssetParam, *usecase.Operator) (*asset.Asset, error)
	Remove(context.Context, id.AssetID, *usecase.Operator) (id.AssetID, error)
}
