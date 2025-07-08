package memory

import (
	"context"
	"sort"
	"strings"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
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

func (r *Asset) FindByProject(_ context.Context, pid id.ProjectID, filter repo.AssetFilter) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
	result := r.data.FindAll(func(k id.AssetID, v *asset.Asset) bool {
		return v.Project() == pid && (filter.Keyword == nil || strings.Contains(v.Name(), *filter.Keyword))
	})

	if filter.Sort != nil {
		s := *filter.Sort
		sort.SliceStable(result, func(i, j int) bool {
			if s.Key == "id" {
				return result[i].ID().Compare(result[j].ID()) < 0
			}
			if s.Key == "size" {
				return result[i].Size() < result[j].Size()
			}
			if s.Key == "name" {
				return strings.Compare(result[i].Name(), result[j].Name()) < 0
			}
			return false
		})
	}

	total := int64(len(result))
	if total == 0 {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	if filter.Pagination != nil && filter.Pagination.Page != nil {
		// Page-based pagination
		skip := (filter.Pagination.Page.Page - 1) * filter.Pagination.Page.PageSize
		if skip >= len(result) {
			return nil, interfaces.NewPageBasedInfo(total, filter.Pagination.Page.Page, filter.Pagination.Page.PageSize), nil
		}

		end := skip + filter.Pagination.Page.PageSize
		if end > len(result) {
			end = len(result)
		}

		return result[skip:end], interfaces.NewPageBasedInfo(total, filter.Pagination.Page.Page, filter.Pagination.Page.PageSize), nil
	}

	return result, interfaces.NewPageBasedInfo(total, 1, int(total)), nil
}

func (r *Asset) TotalSizeByProject(_ context.Context, pid id.ProjectID) (t uint64, err error) {
	r.data.Range(func(k id.AssetID, v *asset.Asset) bool {
		if v.Project() == pid {
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
