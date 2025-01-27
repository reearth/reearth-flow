package memory

import (
	"context"
	"sort"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
)

type Deployment struct {
	lock sync.Mutex
	data map[id.DeploymentID]*deployment.Deployment
	f    repo.WorkspaceFilter
}

func NewDeployment() *Deployment {
	return &Deployment{
		data: map[id.DeploymentID]*deployment.Deployment{},
	}
}

func (r *Deployment) Filtered(f repo.WorkspaceFilter) repo.Deployment {
	return &Deployment{
		// note data is shared between the source repo and mutex cannot work well
		data: r.data,
		f:    r.f.Merge(f),
	}
}

func (r *Deployment) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*deployment.Deployment, *usecasex.PageInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, nil, nil
	}

	// Pre-allocate slice with estimated capacity
	result := make([]*deployment.Deployment, 0, len(r.data))
	for _, d := range r.data {
		if d.Workspace() == id {
			result = append(result, d)
		}
	}

	total := int64(len(result))
	if total == 0 {
		return nil, &usecasex.PageInfo{TotalCount: 0}, nil
	}

	// Apply sorting
	direction := 1 // default ascending
	if pagination != nil && pagination.Page != nil && pagination.Page.OrderDir != nil && *pagination.Page.OrderDir == "DESC" {
		direction = -1
	}

	sort.Slice(result, func(i, j int) bool {
		if pagination != nil && pagination.Page != nil && pagination.Page.OrderBy != nil && *pagination.Page.OrderBy == "version" {
			if direction == 1 {
				return result[i].Version() < result[j].Version()
			}
			return result[i].Version() > result[j].Version()
		}
		if direction == 1 {
			return result[i].UpdatedAt().Before(result[j].UpdatedAt())
		}
		return result[i].UpdatedAt().After(result[j].UpdatedAt())
	})

	// Handle pagination
	if pagination == nil {
		return result, &usecasex.PageInfo{TotalCount: total}, nil
	}

	if pagination.Page != nil {
		// Page-based pagination
		skip := (pagination.Page.Page - 1) * pagination.Page.PageSize
		if skip >= len(result) {
			return nil, &usecasex.PageInfo{
				TotalCount:      total,
				HasNextPage:     false,
				HasPreviousPage: false,
			}, nil
		}

		end := skip + pagination.Page.PageSize
		if end > len(result) {
			end = len(result)
		}

		// Calculate if there is a next page
		hasNextPage := end < len(result)
		// Calculate if there is a previous page
		hasPreviousPage := pagination.Page.Page > 1 && skip < len(result)

		// Get the current page
		pageResult := result[skip:end]

		return pageResult, &usecasex.PageInfo{
			TotalCount:      total,
			HasNextPage:     hasNextPage,
			HasPreviousPage: hasPreviousPage,
		}, nil
	}

	if pagination.Cursor != nil {
		// Cursor-based pagination
		var start int64
		if pagination.Cursor.Cursor.After != nil {
			// Find the position of the "after" cursor
			afterID := string(*pagination.Cursor.Cursor.After)
			for i, d := range result {
				if d.ID().String() == afterID {
					start = int64(i + 1)
					break
				}
			}
		}

		end := total
		if pagination.Cursor.Cursor.First != nil {
			end = start + *pagination.Cursor.Cursor.First
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

	return result, &usecasex.PageInfo{TotalCount: total}, nil
}

func (r *Deployment) FindByProject(ctx context.Context, id id.ProjectID) (*deployment.Deployment, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	for _, d := range r.data {
		if d.Project() != nil && *d.Project() == id && r.f.CanRead(d.Workspace()) {
			return d, nil
		}
	}

	return nil, rerror.ErrNotFound
}

func (r *Deployment) FindByID(ctx context.Context, id id.DeploymentID) (*deployment.Deployment, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if p, ok := r.data[id]; ok && r.f.CanRead(p.Workspace()) {
		return p, nil
	}
	return nil, rerror.ErrNotFound
}

func (r *Deployment) FindByIDs(ctx context.Context, ids id.DeploymentIDList) ([]*deployment.Deployment, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	result := []*deployment.Deployment{}
	for _, id := range ids {
		if d, ok := r.data[id]; ok && r.f.CanRead(d.Workspace()) {
			result = append(result, d)
			continue
		}
		result = append(result, nil)
	}
	return result, nil
}

func (r *Deployment) FindByVersion(ctx context.Context, wsID accountdomain.WorkspaceID, projectID *id.ProjectID, version string) (*deployment.Deployment, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(wsID) {
		return nil, nil
	}

	for _, d := range r.data {
		if d.Workspace() == wsID && d.Version() == version {
			if projectID != nil {
				if d.Project() != nil && *d.Project() == *projectID {
					return d, nil
				}
			} else if d.Project() == nil {
				return d, nil
			}
		}
	}

	return nil, rerror.ErrNotFound
}

func (r *Deployment) FindHead(ctx context.Context, wsID accountdomain.WorkspaceID, projectID *id.ProjectID) (*deployment.Deployment, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(wsID) {
		return nil, nil
	}

	for _, d := range r.data {
		if d.Workspace() == wsID && d.IsHead() {
			if projectID != nil {
				if d.Project() != nil && *d.Project() == *projectID {
					return d, nil
				}
			} else if d.Project() == nil {
				return d, nil
			}
		}
	}

	return nil, rerror.ErrNotFound
}

func (r *Deployment) FindVersions(ctx context.Context, wsID accountdomain.WorkspaceID, projectID *id.ProjectID) ([]*deployment.Deployment, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(wsID) {
		return nil, nil
	}

	var result []*deployment.Deployment
	for _, d := range r.data {
		if d.Workspace() == wsID {
			if projectID != nil {
				if d.Project() != nil && *d.Project() == *projectID {
					result = append(result, d)
				}
			} else if d.Project() == nil {
				result = append(result, d)
			}
		}
	}

	sort.Slice(result, func(i, j int) bool {
		return result[i].Version() > result[j].Version()
	})

	return result, nil
}

func (r *Deployment) Save(ctx context.Context, d *deployment.Deployment) error {
	if !r.f.CanWrite(d.Workspace()) {
		return repo.ErrOperationDenied
	}

	r.lock.Lock()
	defer r.lock.Unlock()

	r.data[d.ID()] = d
	return nil
}

func (r *Deployment) Remove(ctx context.Context, id id.DeploymentID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	if d, ok := r.data[id]; ok && r.f.CanWrite(d.Workspace()) {
		delete(r.data, id)
	}
	return nil
}
