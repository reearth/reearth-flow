package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/util"
)

type Workflow struct {
	data *util.SyncMap[id.WorkflowID, *workflow.Workflow]
	f    repo.WorkspaceFilter
	lock sync.Mutex
}

func NewWorkflow() *Workflow {
	return &Workflow{
		data: util.SyncMapFrom[id.WorkflowID, *workflow.Workflow](nil),
	}
}

func (r *Workflow) Filtered(f repo.WorkspaceFilter) repo.Workflow {
	return &Workflow{
		// note data is shared between the source repo and mutex cannot work well
		data: r.data,
		f:    r.f.Merge(f),
	}
}

func (r *Workflow) FindByID(_ context.Context, wfid id.WorkflowID) (*workflow.Workflow, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	d, ok := r.data.Load(wfid)
	if ok && r.f.CanRead(d.Workspace()) {
		return d, nil
	}

	return nil, rerror.ErrNotFound
}

func (r *Workflow) Save(_ context.Context, w *workflow.Workflow) error {
	if !r.f.CanWrite(w.Workspace()) {
		return repo.ErrOperationDenied
	}

	r.data.Store(w.ID(), w)
	return nil
}

func (r *Workflow) Remove(ctx context.Context, id id.WorkflowID) error {
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
