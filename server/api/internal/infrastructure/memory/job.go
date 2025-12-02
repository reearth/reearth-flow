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
	"github.com/reearth/reearth-flow/api/pkg/job"
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

func (r *Job) FindByWorkspace(ctx context.Context, id accountsid.WorkspaceID, pagination *interfaces.PaginationParam, keyword *string) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	result := []*job.Job{}
	for _, j := range r.data {
		if j.Workspace() != id {
			continue
		}

		if keyword != nil && *keyword != "" {
			if !strings.Contains(strings.ToLower(j.ID().String()), strings.ToLower(*keyword)) {
				continue
			}
		}

		result = append(result, j)
	}

	total := int64(len(result))
	if total == 0 {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	if pagination != nil && pagination.Page != nil {
		field := "startedAt"
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
			case "startedAt":
				ti := result[i].StartedAt()
				tj := result[j].StartedAt()
				if !ti.Equal(tj) {
					return compare(ti.Before(tj))
				}
				return compare(result[i].ID().String() < result[j].ID().String())
			case "completedAt":
				ti := result[i].CompletedAt()
				tj := result[j].CompletedAt()
				if ti == nil && tj == nil {
					return compare(result[i].ID().String() < result[j].ID().String())
				}
				if ti == nil {
					return compare(true)
				}
				if tj == nil {
					return compare(false)
				}
				if !ti.Equal(*tj) {
					return compare(ti.Before(*tj))
				}
				return compare(result[i].ID().String() < result[j].ID().String())
			case "status":
				si := result[i].Status()
				sj := result[j].Status()
				if si != sj {
					return compare(si < sj)
				}
				return compare(result[i].ID().String() < result[j].ID().String())
			default:
				ti := result[i].StartedAt()
				tj := result[j].StartedAt()
				if !ti.Equal(tj) {
					return compare(ti.Before(tj))
				}
				return compare(result[i].ID().String() < result[j].ID().String())
			}
		})

		// Apply pagination
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
