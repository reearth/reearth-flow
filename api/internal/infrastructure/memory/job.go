package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type Job struct {
	lock sync.Mutex
	data map[id.JobID]*job.Job
	f    repo.WorkspaceFilter
}

func NewJob() *Job {
	return &Job{
		data: map[id.JobID]*job.Job{},
	}
}

func (r *Job) Filtered(f repo.WorkspaceFilter) repo.Job {
	return &Job{
		data: r.data,
		f:    r.f.Merge(f),
	}
}

func (r *Job) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, p *usecasex.Pagination) ([]*job.Job, *usecasex.PageInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, nil, nil
	}

	result := []*job.Job{}
	for _, j := range r.data {
		if j.Workspace() == id {
			result = append(result, j)
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

func (r *Job) FindByID(ctx context.Context, id id.JobID) (*job.Job, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if j, ok := r.data[id]; ok && r.f.CanRead(j.Workspace()) {
		return j, nil
	}
	return nil, rerror.ErrNotFound
}

func (r *Job) FindByIDs(ctx context.Context, ids id.JobIDList) ([]*job.Job, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	result := []*job.Job{}
	for _, id := range ids {
		if j, ok := r.data[id]; ok && r.f.CanRead(j.Workspace()) {
			result = append(result, j)
			continue
		}
		result = append(result, nil)
	}
	return result, nil
}

func (r *Job) Save(ctx context.Context, j *job.Job) error {
	log.Debugfc(ctx, "Saving job - ID: ")
	if !r.f.CanWrite(j.Workspace()) {
		return repo.ErrOperationDenied
	}

	r.lock.Lock()
	defer r.lock.Unlock()

	r.data[j.ID()] = j
	return nil
}

func (r *Job) Remove(ctx context.Context, id id.JobID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	if j, ok := r.data[id]; ok && r.f.CanWrite(j.Workspace()) {
		delete(r.data, id)
	}
	return nil
}
