package memory

import (
	"context"
	"sort"
	"sync"

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

func (r *Deployment) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, p *usecasex.Pagination) ([]*deployment.Deployment, *usecasex.PageInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(id) {
		return nil, nil, nil
	}

	result := []*deployment.Deployment{}
	for _, d := range r.data {
		if d.Workspace() == id {
			result = append(result, d)
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
