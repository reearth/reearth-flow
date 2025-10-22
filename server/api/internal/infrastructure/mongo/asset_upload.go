package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearthx/mongox"
	"go.mongodb.org/mongo-driver/bson"
)

var (
	assetUploadIndexes       = []string{"expires_at"}
	assetUploadUniqueIndexes = []string{"uuid"}
)

type AssetUpload struct {
	client *mongox.ClientCollection
	f      repo.WorkspaceFilter
}

func NewAssetUpload(client *mongox.Client) *AssetUpload {
	return &AssetUpload{client: client.WithCollection("asset_upload")}
}

func (r *AssetUpload) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, assetUploadIndexes, assetUploadUniqueIndexes)
}

func (r *AssetUpload) Filtered(f repo.WorkspaceFilter) repo.AssetUpload {
	return &AssetUpload{
		client: r.client,
		f:      r.f.Merge(f),
	}
}

func (r *AssetUpload) FindByID(ctx context.Context, uuid string) (*asset.Upload, error) {
	return r.findOne(ctx, bson.M{
		"uuid": uuid,
	})
}

func (r *AssetUpload) Save(ctx context.Context, upload *asset.Upload) error {
	if !r.f.CanWrite(upload.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewAssetUpload(upload)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *AssetUpload) findOne(ctx context.Context, filter any) (*asset.Upload, error) {
	c := mongodoc.NewAssetUploadConsumer(r.f.Readable)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}
