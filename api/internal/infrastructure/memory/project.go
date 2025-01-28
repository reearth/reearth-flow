package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
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

func (r *Project) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, p *usecasex.Pagination) ([]*project.Project, *usecasex.PageInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, nil, nil
	}

	result := []*project.Project{}
	for _, d := range r.data {
		if d.Workspace() == id {
			result = append(result, d)
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
