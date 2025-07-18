package mongo

import (
	"context"
	"fmt"
	"regexp"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo/options"
)

var (
	assetIndexes       = []string{"workspace"}
	assetUniqueIndexes = []string{"id"}
)

type Asset struct {
	client *mongox.ClientCollection
	f      repo.WorkspaceFilter
}

func NewAsset(client *mongox.Client) *Asset {
	return &Asset{client: client.WithCollection("asset")}
}

func (r *Asset) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, assetIndexes, assetUniqueIndexes)
}

func (r *Asset) Filtered(f repo.WorkspaceFilter) repo.Asset {
	return &Asset{
		client: r.client,
		f:      r.f.Merge(f),
	}
}

func (r *Asset) FindByID(ctx context.Context, id id.AssetID) (*asset.Asset, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	})
}

func (r *Asset) FindByIDs(ctx context.Context, ids id.AssetIDList) ([]*asset.Asset, error) {
	if len(ids) == 0 {
		return nil, nil
	}

	res, err := r.find(ctx, bson.M{
		"id": bson.M{"$in": ids.Strings()},
	})
	if err != nil {
		return nil, err
	}
	return filterAssets(ids, res), nil
}

func (r *Asset) FindByWorkspace(ctx context.Context, wid accountdomain.WorkspaceID, uFilter repo.AssetFilter) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
	var filter any = bson.M{
		"workspace": wid.String(),
	}

	if uFilter.Keyword != nil {
		filter = mongox.And(filter, "name", bson.M{
			"$regex": primitive.Regex{Pattern: fmt.Sprintf(".*%s.*", regexp.QuoteMeta(*uFilter.Keyword)), Options: "i"},
		})
	}

	return r.paginate(ctx, filter, uFilter.Sort, uFilter.Pagination)
}

func (r *Asset) TotalSizeByWorkspace(ctx context.Context, wid accountdomain.WorkspaceID) (uint64, error) {
	c, err := r.client.Client().Aggregate(ctx, []bson.M{
		{"$match": bson.M{"workspace": wid.String()}},
		{"$group": bson.M{"_id": nil, "size": bson.M{"$sum": "$size"}}},
	})
	if err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, err)
	}
	defer func() {
		_ = c.Close(ctx)
	}()

	if !c.Next(ctx) {
		return 0, nil
	}

	type resp struct {
		Size uint64
	}
	var res resp
	if err := c.Decode(&res); err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, err)
	}
	return res.Size, nil
}

func (r *Asset) Save(ctx context.Context, asset *asset.Asset) error {
	if !r.f.CanWrite(asset.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewAsset(asset)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *Asset) Delete(ctx context.Context, id id.AssetID) error {
	return r.client.RemoveOne(ctx, r.writeFilter(bson.M{
		"id": id.String(),
	}))
}

func (r *Asset) paginate(ctx context.Context, filter any, sort *asset.SortType, pagination *interfaces.PaginationParam) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
	c := mongodoc.NewAssetConsumer(r.f.Readable)

	if pagination != nil && pagination.Page != nil {
		skip := (pagination.Page.Page - 1) * pagination.Page.PageSize
		limit := pagination.Page.PageSize

		total, err := r.client.Count(ctx, filter)
		if err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		opts := options.Find()
		if sort != nil {
			opts.SetSort(bson.D{{Key: sort.Key, Value: 1}})
		}

		opts.SetSkip(int64(skip)).SetLimit(int64(limit))

		if err := r.client.Find(ctx, filter, c, opts); err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		return c.Result, interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize), nil
	}

	return c.Result, interfaces.NewPageBasedInfo(int64(len(c.Result)), 1, len(c.Result)), nil
}

func (r *Asset) find(ctx context.Context, filter any) ([]*asset.Asset, error) {
	c := mongodoc.NewAssetConsumer(r.f.Readable)
	if err2 := r.client.Find(ctx, filter, c); err2 != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, err2)
	}
	return c.Result, nil
}

func (r *Asset) findOne(ctx context.Context, filter any) (*asset.Asset, error) {
	c := mongodoc.NewAssetConsumer(r.f.Readable)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}

func filterAssets(ids []id.AssetID, rows []*asset.Asset) []*asset.Asset {
	res := make([]*asset.Asset, 0, len(ids))
	for _, id := range ids {
		var r2 *asset.Asset
		for _, r := range rows {
			if r.ID() == id {
				r2 = r
				break
			}
		}
		res = append(res, r2)
	}
	return res
}

// func (r *Asset) readFilter(filter any) any {
// 	return applyWorkspaceFilter(filter, r.f.Readable)
// }

func (r *Asset) writeFilter(filter any) any {
	return applyWorkspaceFilter(filter, r.f.Writable)
}
