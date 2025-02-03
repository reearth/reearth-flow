package memory

import (
	"context"
	"sort"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
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

func (r *Job) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*job.Job, *interfaces.PageBasedInfo, error) {
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
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
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
			case "startedAt":
				ti := result[i].StartedAt()
				tj := result[j].StartedAt()
				if !ti.Equal(tj) {
					if direction == 1 {
						return ti.Before(tj)
					}
					return ti.After(tj)
				}
			case "completedAt":
				ti := result[i].CompletedAt()
				tj := result[j].CompletedAt()
				if ti == nil && tj == nil {
					return result[i].ID().String() < result[j].ID().String()
				}
				if ti == nil {
					return direction == 1
				}
				if tj == nil {
					return direction != 1
				}
				if !ti.Equal(*tj) {
					if direction == 1 {
						return ti.Before(*tj)
					}
					return ti.After(*tj)
				}
			}
		}
		// Default sort or tie-breaker: by ID
		return result[i].ID().String() < result[j].ID().String()
	})

	if pagination == nil {
		return result, interfaces.NewPageBasedInfo(total, 1, int(total)), nil
	}

	if pagination.Page != nil {
		// Page-based pagination
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
