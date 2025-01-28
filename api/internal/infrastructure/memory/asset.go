package memory

import (
	"context"
	"sort"
	"strings"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/reearth/reearthx/util"
)

type Asset struct {
	data *util.SyncMap[id.AssetID, *asset.Asset]
	f    repo.WorkspaceFilter
}

func NewAsset() *Asset {
	return &Asset{
		data: util.SyncMapFrom[id.AssetID, *asset.Asset](nil),
	}
}

func (r *Asset) Filtered(f repo.WorkspaceFilter) repo.Asset {
	return &Asset{
		data: r.data,
		f:    r.f.Merge(f),
	}
}

func (r *Asset) FindByID(_ context.Context, id id.AssetID) (*asset.Asset, error) {
	d, ok := r.data.Load(id)
	if ok && r.f.CanRead(d.Workspace()) {
		return d, nil
	}
	return &asset.Asset{}, rerror.ErrNotFound
}

func (r *Asset) FindByIDs(_ context.Context, ids id.AssetIDList) ([]*asset.Asset, error) {
	return r.data.FindAll(func(k id.AssetID, v *asset.Asset) bool {
		return ids.Has(k) && r.f.CanRead(v.Workspace())
	}), nil
}

func (r *Asset) FindByWorkspace(_ context.Context, wid accountdomain.WorkspaceID, filter repo.AssetFilter) ([]*asset.Asset, *usecasex.PageInfo, error) {
	if !r.f.CanRead(wid) {
		return nil, usecasex.EmptyPageInfo(), nil
	}

	result := r.data.FindAll(func(k id.AssetID, v *asset.Asset) bool {
		return v.Workspace() == wid && (filter.Keyword == nil || strings.Contains(v.Name(), *filter.Keyword))
	})

	if filter.Sort != nil {
		s := *filter.Sort
		sort.SliceStable(result, func(i, j int) bool {
			if s == asset.SortTypeID {
				return result[i].ID().Compare(result[j].ID()) < 0
			}
			if s == asset.SortTypeSize {
				return result[i].Size() < result[j].Size()
			}
			if s == asset.SortTypeName {
				return strings.Compare(result[i].Name(), result[j].Name()) < 0
			}
			return false
		})
	}

	total := int64(len(result))
	if total == 0 {
		return nil, &usecasex.PageInfo{TotalCount: 0}, nil
	}

	if filter.Pagination != nil {
		if filter.Pagination.Cursor != nil {
			// Cursor-based pagination
			var start int64
			if filter.Pagination.Cursor.After != nil {
				afterID := string(*filter.Pagination.Cursor.After)
				for i, d := range result {
					if d.ID().String() == afterID {
						start = int64(i + 1)
						break
					}
				}
			}

			end := total
			if filter.Pagination.Cursor.First != nil {
				end = start + *filter.Pagination.Cursor.First
				if end > total {
					end = total
				}
			}

			if start >= total {
				return nil, &usecasex.PageInfo{
					TotalCount:      total,
					HasNextPage:     false,
					HasPreviousPage: start > 0,
				}, nil
			}

			var startCursor, endCursor *usecasex.Cursor
			if start < end {
				sc := usecasex.Cursor(result[start].ID().String())
				ec := usecasex.Cursor(result[end-1].ID().String())
				startCursor = &sc
				endCursor = &ec
			}

			return result[start:end], &usecasex.PageInfo{
				TotalCount:      total,
				HasNextPage:     end < total,
				HasPreviousPage: start > 0,
				StartCursor:     startCursor,
				EndCursor:       endCursor,
			}, nil
		} else if filter.Pagination.Offset != nil {
			// Page-based pagination
			skip := int(filter.Pagination.Offset.Offset)
			limit := int(filter.Pagination.Offset.Limit)
			if skip >= len(result) {
				pageInfo := interfaces.NewPageBasedInfo(total, skip/limit+1, limit)
				return nil, pageInfo.ToPageInfo(), nil
			}

			end := skip + limit
			if end > len(result) {
				end = len(result)
			}

			pageInfo := interfaces.NewPageBasedInfo(total, skip/limit+1, limit)
			return result[skip:end], pageInfo.ToPageInfo(), nil
		}
	}

	return result, &usecasex.PageInfo{
		TotalCount: total,
	}, nil
}

func (r *Asset) TotalSizeByWorkspace(_ context.Context, wid accountdomain.WorkspaceID) (t int64, err error) {
	if !r.f.CanRead(wid) {
		return 0, nil
	}

	r.data.Range(func(k id.AssetID, v *asset.Asset) bool {
		if v.Workspace() == wid {
			t += v.Size()
		}
		return true
	})
	return
}

func (r *Asset) Save(_ context.Context, a *asset.Asset) error {
	if !r.f.CanWrite(a.Workspace()) {
		return repo.ErrOperationDenied
	}

	r.data.Store(a.ID(), a)
	return nil
}

func (r *Asset) Remove(_ context.Context, id id.AssetID) error {
	a, _ := r.data.Load(id)
	if a == nil {
		return nil
	}

	if !r.f.CanWrite(a.Workspace()) {
		return repo.ErrOperationDenied
	}

	r.data.Delete(id)
	return nil
}
