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

	total := int64(len(result))
	if total == 0 {
		return nil, &usecasex.PageInfo{TotalCount: 0}, nil
	}

	if p != nil && p.Cursor != nil {
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
	}

	return result, &usecasex.PageInfo{
		TotalCount: total,
	}, nil
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
