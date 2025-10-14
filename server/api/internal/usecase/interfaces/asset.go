package interfaces

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type AssetFilterType string

const (
	AssetFilterDate AssetFilterType = "DATE"
	AssetFilterSize AssetFilterType = "SIZE"
	AssetFilterName AssetFilterType = "NAME"
)

type CreateAssetParam struct {
	WorkspaceID id.WorkspaceID
	File        *file.File
	Name        *string
}

type UpdateAssetParam struct {
	AssetID id.AssetID
	Name    *string
}

type CreateAssetUploadParam struct {
	WorkspaceID     id.WorkspaceID
	Filename        string
	ContentLength   int64
	ContentType     string
	ContentEncoding string
	Cursor          string
}

type AssetUpload struct {
	URL             string
	UUID            string
	ContentType     string
	ContentLength   int64
	ContentEncoding string
	Next            string
}

var ErrCreateAssetFailed error = errors.New("failed to create asset")

type Asset interface {
	Fetch(context.Context, []id.AssetID) ([]*asset.Asset, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *string, *asset.SortType, *PaginationParam) ([]*asset.Asset, *PageBasedInfo, error)
	Create(context.Context, CreateAssetParam) (*asset.Asset, error)
	Update(context.Context, UpdateAssetParam) (*asset.Asset, error)
	Delete(context.Context, id.AssetID) (id.AssetID, error)
	CreateUpload(context.Context, CreateAssetUploadParam, *usecase.Operator) (*AssetUpload, error)
}
