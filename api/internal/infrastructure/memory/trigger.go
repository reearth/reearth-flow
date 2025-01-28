package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type Trigger struct {
	lock sync.Mutex
	data map[id.TriggerID]*trigger.Trigger
	f    repo.WorkspaceFilter
}

func NewTrigger() *Trigger {
	return &Trigger{
		data: map[id.TriggerID]*trigger.Trigger{},
	}
}

func (r *Trigger) Filtered(f repo.WorkspaceFilter) repo.Trigger {
	return &Trigger{
		data: r.data,
		f:    r.f.Merge(f),
	}
}

func (r *Trigger) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, p *usecasex.Pagination) ([]*trigger.Trigger, *usecasex.PageInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, nil, nil
	}

	result := []*trigger.Trigger{}
	for _, t := range r.data {
		if t.Workspace() == id {
			result = append(result, t)
		}
	}

	total := int64(len(result))
	if total == 0 {
		return nil, &usecasex.PageInfo{TotalCount: 0}, nil
	}

	if p != nil {
		if p.Cursor != nil {
			// Cursor-based pagination
			var start int64
			if p.Cursor.After != nil {
				afterID := string(*p.Cursor.After)
				for i, d := range result {
					if d.ID().String() == afterID {
						start = int64(i + 1)
						break
					}
				}
			}

			end := total
			if p.Cursor.First != nil {
				end = start + *p.Cursor.First
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
		} else if p.Offset != nil {
			// Page-based pagination
			skip := int(p.Offset.Offset)
			limit := int(p.Offset.Limit)

			if skip >= int(total) {
				pageInfo := interfaces.NewPageBasedInfo(total, skip/limit+1, limit)
				return nil, pageInfo.ToPageInfo(), nil
			}

			end := skip + limit
			if end > int(total) {
				end = int(total)
			}

			pageInfo := interfaces.NewPageBasedInfo(total, skip/limit+1, limit)
			return result[skip:end], pageInfo.ToPageInfo(), nil
		}
	}

	return result, &usecasex.PageInfo{
		TotalCount: total,
	}, nil
}

func (r *Trigger) FindByID(ctx context.Context, id id.TriggerID) (*trigger.Trigger, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if t, ok := r.data[id]; ok && r.f.CanRead(t.Workspace()) {
		return t, nil
	}
	return nil, rerror.ErrNotFound
}

func (r *Trigger) FindByIDs(ctx context.Context, ids id.TriggerIDList) ([]*trigger.Trigger, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	result := make([]*trigger.Trigger, len(ids))
	for i, id := range ids {
		if t, ok := r.data[id]; ok && r.f.CanRead(t.Workspace()) {
			result[i] = t
			continue
		}
		result[i] = nil
	}
	return result, nil
}

func (r *Trigger) Save(ctx context.Context, t *trigger.Trigger) error {
	if !r.f.CanWrite(t.Workspace()) {
		return repo.ErrOperationDenied
	}

	r.lock.Lock()
	defer r.lock.Unlock()

	r.data[t.ID()] = t
	return nil
}

func (r *Trigger) Remove(ctx context.Context, id id.TriggerID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	if t, ok := r.data[id]; ok && r.f.CanWrite(t.Workspace()) {
		delete(r.data, id)
	}
	return nil
}
