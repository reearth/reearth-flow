package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/rerror"
)

type Workflow struct {
	lock sync.Mutex
	data map[id.WorkflowID]*workflow.Workflow
	f    repo.WorkspaceFilter
}

func NewWorkflow() repo.Workflow {
	return &Workflow{
		data: map[id.WorkflowID]*workflow.Workflow{},
	}
}

func (r *Workflow) Filtered(f repo.WorkspaceFilter) repo.Workflow {
	return &Workflow{
		// note data is shared between the source repo and mutex cannot work well
		data: r.data,
		f:    r.f.Merge(f),
	}
}

func (r *Workflow) FindByID(ctx context.Context, wsid accountdomain.WorkspaceID, wfid id.WorkflowID) (*workflow.Workflow, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if w, ok := r.data[wfid]; ok && r.f.CanRead(wsid) {
		return w, nil
	}
	return nil, rerror.ErrNotFound
}

func (r *Workflow) Save(ctx context.Context, wsid accountdomain.WorkspaceID, w *workflow.Workflow) error {
	if !r.f.CanWrite(wsid) {
		return repo.ErrOperationDenied
	}

	r.lock.Lock()
	defer r.lock.Unlock()

	r.data[w.ID] = w
	return nil
}

func (r *Workflow) Remove(ctx context.Context, wsid accountdomain.WorkspaceID, id id.WorkflowID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	if _, ok := r.data[id]; ok && r.f.CanRead(wsid) {
		delete(r.data, id)
	}
	return nil
}
