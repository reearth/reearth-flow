package memory

import (
	"context"
	"sort"
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

func (r *Trigger) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*trigger.Trigger, *usecasex.PageInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, nil, nil
	}

	// Pre-allocate slice with estimated capacity
	result := make([]*trigger.Trigger, 0, len(r.data))
	for _, t := range r.data {
		if t.Workspace() == id {
			result = append(result, t)
		}
	}

	total := int64(len(result))
	if total == 0 {
		return nil, &usecasex.PageInfo{TotalCount: 0}, nil
	}

	// Apply sorting
	direction := 1 // default ascending
	if pagination != nil && pagination.Page != nil && pagination.Page.OrderDir != nil && *pagination.Page.OrderDir == "DESC" {
		direction = -1
	}

	sort.Slice(result, func(i, j int) bool {
		if pagination != nil && pagination.Page != nil && pagination.Page.OrderBy != nil {
			// Compare by specified field
			switch *pagination.Page.OrderBy {
			case "createdAt":
				ti, tj := result[i].CreatedAt(), result[j].CreatedAt()
				if !ti.Equal(tj) {
					if direction == 1 {
						return ti.Before(tj)
					}
					return ti.After(tj)
				}
			}
		}
		// Default sort or tie-breaker: by ID
		return result[i].ID().String() < result[j].ID().String()
	})

	// Handle pagination
	if pagination == nil {
		return result, &usecasex.PageInfo{TotalCount: total}, nil
	}

	if pagination.Page != nil {
		// Page-based pagination
		skip := (pagination.Page.Page - 1) * pagination.Page.PageSize
		if skip >= len(result) {
			return nil, interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize).ToPageInfo(), nil
		}

		end := skip + pagination.Page.PageSize
		if end > len(result) {
			end = len(result)
		}

		// Get the current page
		pageResult := result[skip:end]

		// Create page-based info
		pageInfo := interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize)

		return pageResult, pageInfo.ToPageInfo(), nil
	}

	return result, &usecasex.PageInfo{TotalCount: total}, nil
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
