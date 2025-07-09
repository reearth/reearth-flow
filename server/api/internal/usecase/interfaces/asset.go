package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type AssetFilterType string

const (
	AssetFilterDate AssetFilterType = "DATE"
	AssetFilterSize AssetFilterType = "SIZE"
	AssetFilterName AssetFilterType = "NAME"
)

type CreateAssetParam struct {
	WorkspaceID accountdomain.WorkspaceID
	UserID      accountdomain.UserID
	File        *file.File
	Name        *string
}

type UpdateAssetParam struct {
	AssetID id.AssetID
	Name    *string
}

var ErrCreateAssetFailed error = errors.New("failed to create asset")

type Asset interface {
	Fetch(context.Context, []id.AssetID) ([]*asset.Asset, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *string, *asset.SortType, *PaginationParam) ([]*asset.Asset, *PageBasedInfo, error)
	Create(context.Context, CreateAssetParam) (*asset.Asset, error)
	Update(context.Context, UpdateAssetParam) (*asset.Asset, error)
	Delete(context.Context, id.AssetID) (id.AssetID, error)
}
