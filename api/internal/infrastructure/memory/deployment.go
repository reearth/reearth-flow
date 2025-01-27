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

	result := make([]*deployment.Deployment, 0, len(r.data))
	for _, d := range r.data {
		if d.Workspace() == id {
			result = append(result, d)
		}
	}

	if pagination != nil {
		if pagination.Page != nil {
			// Page-based pagination
			skip := (pagination.Page.Page - 1) * pagination.Page.PageSize
			limit := pagination.Page.PageSize

			if skip >= len(result) {
				return nil, &usecasex.PageInfo{
					TotalCount: int64(len(result)),
				}, nil
			}

			end := skip + limit
			if end > len(result) {
				end = len(result)
			}

			return result[skip:end], &usecasex.PageInfo{
				TotalCount:      int64(len(result)),
				HasNextPage:     end < len(result),
				HasPreviousPage: skip > 0,
			}, nil
		} else if pagination.Cursor != nil {
			// Cursor-based pagination
			var startIndex int
			endIndex := len(result)

			if pagination.Cursor != nil && pagination.Cursor.Cursor != nil {
				cursor := pagination.Cursor.Cursor
				if cursor.First != nil {
					endIndex = int(*cursor.First)
					if endIndex > len(result) {
						endIndex = len(result)
					}
				}

				if cursor.After != nil {
					for i, d := range result {
						if usecasex.Cursor(d.ID().String()) == *cursor.After {
							startIndex = i + 1
							break
						}
					}
				}
			}

			if startIndex >= len(result) {
				return nil, &usecasex.PageInfo{
					TotalCount: int64(len(result)),
				}, nil
			}

			if startIndex > endIndex {
				endIndex = startIndex
			}

			slicedResult := result[startIndex:endIndex]

			var startCursor, endCursor *usecasex.Cursor
			if len(slicedResult) > 0 {
				start := usecasex.Cursor(slicedResult[0].ID().String())
				end := usecasex.Cursor(slicedResult[len(slicedResult)-1].ID().String())
				startCursor = &start
				endCursor = &end
			}

			return slicedResult, &usecasex.PageInfo{
				TotalCount:      int64(len(result)),
				HasNextPage:     endIndex < len(result),
				HasPreviousPage: startIndex > 0,
				StartCursor:     startCursor,
				EndCursor:       endCursor,
			}, nil
		}
	}

	return result, &usecasex.PageInfo{
		TotalCount: int64(len(result)),
	}, nil
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
