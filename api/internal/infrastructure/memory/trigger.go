package memory

import (
	"context"
	"sync"

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
	for _, d := range r.data {
		if d.Workspace() == id {
			result = append(result, d)
		}
	}

	var startCursor, endCursor *usecasex.Cursor
	if len(result) > 0 {
		_startCursor := usecasex.Cursor(result[0].ID().String())
		_endCursor := usecasex.Cursor(result[len(result)-1].ID().String())
		startCursor = &_startCursor
		endCursor = &_endCursor
	}

	return result, usecasex.NewPageInfo(
		int64(len(result)),
		startCursor,
		endCursor,
		true,
		true,
	), nil
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
