package memory

import (
	"context"
	"sort"
	"strings"
	"sync"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/rerror"
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

func (r *Trigger) FindByWorkspace(ctx context.Context, id accountsid.WorkspaceID, pagination *interfaces.PaginationParam, keyword *string) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	result := make([]*trigger.Trigger, 0, len(r.data))
	for _, t := range r.data {
		if t.Workspace() != id {
			continue
		}

		if keyword != nil && *keyword != "" {
			if !strings.Contains(strings.ToLower(t.Description()), strings.ToLower(*keyword)) &&
				!strings.Contains(strings.ToLower(t.ID().String()), strings.ToLower(*keyword)) {
				continue
			}
		}

		result = append(result, t)
	}

	total := int64(len(result))
	if total == 0 {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	if pagination != nil && pagination.Page != nil {
		field := "createdAt"
		if pagination.Page.OrderBy != nil {
			field = *pagination.Page.OrderBy
		}

		ascending := false
		if pagination.Page.OrderDir != nil && *pagination.Page.OrderDir == "ASC" {
			ascending = true
		}

		sort.Slice(result, func(i, j int) bool {
			compare := func(less bool) bool {
				if ascending {
					return less
				}
				return !less
			}

			switch field {
			case "createdAt":
				ti, tj := result[i].CreatedAt(), result[j].CreatedAt()
				if !ti.Equal(tj) {
					return compare(ti.Before(tj))
				}
				return compare(result[i].ID().String() < result[j].ID().String())
			case "updatedAt":
				ti, tj := result[i].UpdatedAt(), result[j].UpdatedAt()
				if !ti.Equal(tj) {
					return compare(ti.Before(tj))
				}
				return compare(result[i].ID().String() < result[j].ID().String())
			default:
				ti, tj := result[i].CreatedAt(), result[j].CreatedAt()
				if !ti.Equal(tj) {
					return compare(ti.Before(tj))
				}
				return compare(result[i].ID().String() < result[j].ID().String())
			}
		})

		skip := (pagination.Page.Page - 1) * pagination.Page.PageSize
		if skip >= len(result) {
			return nil, interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize), nil
		}

		end := skip + pagination.Page.PageSize
		if end > len(result) {
			end = len(result)
		}

		return result[skip:end], interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize), nil
	}

	return result, interfaces.NewPageBasedInfo(total, 1, int(total)), nil
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

func (r *Trigger) FindByDeployment(ctx context.Context, deploymentID id.DeploymentID) ([]*trigger.Trigger, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	result := make([]*trigger.Trigger, 0)
	for _, t := range r.data {
		if t.Deployment() == deploymentID && r.f.CanRead(t.Workspace()) {
			result = append(result, t)
		}
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
