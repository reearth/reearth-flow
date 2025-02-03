package memory

import (
	"context"
	"sort"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/rerror"
)

type Project struct {
	lock sync.Mutex
	data map[id.ProjectID]*project.Project
	f    repo.WorkspaceFilter
}

func NewProject() repo.Project {
	return &Project{
		data: map[id.ProjectID]*project.Project{},
	}
}

func (r *Project) Filtered(f repo.WorkspaceFilter) repo.Project {
	return &Project{
		// note data is shared between the source repo and mutex cannot work well
		data: r.data,
		f:    r.f.Merge(f),
	}
}

func (r *Project) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	result := make([]*project.Project, 0, len(r.data))
	for _, p := range r.data {
		if p.Workspace() == id {
			result = append(result, p)
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
			switch *pagination.Page.OrderBy {
			case "name":
				ni, nj := result[i].Name(), result[j].Name()
				if ni != nj {
					if direction == 1 {
						return ni < nj
					}
					return ni > nj
				}
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
		return result[i].ID().String() < result[j].ID().String()
	})

	if pagination == nil {
		return result, interfaces.NewPageBasedInfo(total, 1, int(total)), nil
	}

	if pagination.Page != nil {
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

func (r *Project) FindIDsByWorkspace(ctx context.Context, id accountdomain.WorkspaceID) (res []project.ID, _ error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, nil
	}

	for _, d := range r.data {
		if d.Workspace() == id {
			res = append(res, d.ID())
		}
	}
	return
}

func (r *Project) FindByIDs(ctx context.Context, ids id.ProjectIDList) ([]*project.Project, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	result := []*project.Project{}
	for _, id := range ids {
		if d, ok := r.data[id]; ok && r.f.CanRead(d.Workspace()) {
			result = append(result, d)
			continue
		}
		result = append(result, nil)
	}
	return result, nil
}

func (r *Project) FindByID(ctx context.Context, id id.ProjectID) (*project.Project, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if p, ok := r.data[id]; ok && r.f.CanRead(p.Workspace()) {
		return p, nil
	}
	return nil, rerror.ErrNotFound
}

func (r *Project) CountByWorkspace(_ context.Context, ws accountdomain.WorkspaceID) (n int, _ error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	for _, p := range r.data {
		if p.Workspace() == ws && r.f.CanRead(p.Workspace()) {
			n++
		}
	}
	return
}

func (r *Project) CountPublicByWorkspace(_ context.Context, ws accountdomain.WorkspaceID) (n int, _ error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	for _, p := range r.data {
		if p.Workspace() == ws && r.f.CanRead(p.Workspace()) {
			n++
		}
	}
	return
}

func (r *Project) Save(ctx context.Context, p *project.Project) error {
	if !r.f.CanWrite(p.Workspace()) {
		return repo.ErrOperationDenied
	}

	r.lock.Lock()
	defer r.lock.Unlock()

	r.data[p.ID()] = p
	return nil
}

func (r *Project) Remove(ctx context.Context, id id.ProjectID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	if p, ok := r.data[id]; ok && r.f.CanRead(p.Workspace()) {
		delete(r.data, id)
	}
	return nil
}
