package interfaces

import (
	"context"
	"errors"

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
	File        *file.File
	Name        *string
	Token       string
	WorkspaceID id.WorkspaceID
}

type UpdateAssetParam struct {
	Name    *string
	AssetID id.AssetID
}

type CreateAssetUploadParam struct {
	Filename        string
	ContentType     string
	ContentEncoding string
	Cursor          string
	ContentLength   int64
	WorkspaceID     id.WorkspaceID
}

type AssetUpload struct {
	URL             string
	UUID            string
	ContentType     string
	ContentEncoding string
	Next            string
	ContentLength   int64
}

var ErrCreateAssetFailed error = errors.New("failed to create asset")

type Asset interface {
	Fetch(context.Context, []id.AssetID) ([]*asset.Asset, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *string, *asset.SortType, *PaginationParam) ([]*asset.Asset, *PageBasedInfo, error)
	Create(context.Context, CreateAssetParam) (*asset.Asset, error)
	Update(context.Context, UpdateAssetParam) (*asset.Asset, error)
	Delete(context.Context, id.AssetID) (id.AssetID, error)
	CreateUpload(context.Context, CreateAssetUploadParam) (*AssetUpload, error)
}
