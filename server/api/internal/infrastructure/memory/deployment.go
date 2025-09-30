package memory

import (
	"context"
	"sort"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
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
		data: r.data,
		f:    r.f.Merge(f),
	}
}

func (r *Deployment) FindByWorkspace(_ context.Context, wid id.WorkspaceID, p *interfaces.PaginationParam) ([]*deployment.Deployment, *interfaces.PageBasedInfo, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if !r.f.CanRead(wid) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	result := []*deployment.Deployment{}
	for _, d := range r.data {
		if d.Workspace() == wid {
			result = append(result, d)
		}
	}

	total := int64(len(result))
	if total == 0 {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	if p != nil && p.Page != nil {
		field := "updatedAt"
		if p.Page.OrderBy != nil {
			field = *p.Page.OrderBy
		}

		ascending := false
		if p.Page.OrderDir != nil && *p.Page.OrderDir == "ASC" {
			ascending = true
		}

		sort.SliceStable(result, func(i, j int) bool {
			compare := func(less bool) bool {
				if ascending {
					return less
				}
				return !less
			}

			switch field {
			case "updatedAt":
				if result[i].UpdatedAt().Equal(result[j].UpdatedAt()) {
					return result[i].ID().String() < result[j].ID().String()
				}
				return compare(result[i].UpdatedAt().Before(result[j].UpdatedAt()))
			case "description":
				if result[i].Description() == result[j].Description() {
					return result[i].ID().String() < result[j].ID().String()
				}
				return compare(result[i].Description() < result[j].Description())
			case "version":
				if result[i].Version() == result[j].Version() {
					return result[i].ID().String() < result[j].ID().String()
				}
				return compare(result[i].Version() < result[j].Version())
			default:
				if result[i].UpdatedAt().Equal(result[j].UpdatedAt()) {
					return result[i].ID().String() < result[j].ID().String()
				}
				return compare(result[i].UpdatedAt().Before(result[j].UpdatedAt()))
			}
		})

		skip := (p.Page.Page - 1) * p.Page.PageSize
		if skip >= len(result) {
			return nil, interfaces.NewPageBasedInfo(total, p.Page.Page, p.Page.PageSize), nil
		}

		end := skip + p.Page.PageSize
		if end > len(result) {
			end = len(result)
		}

		return result[skip:end], interfaces.NewPageBasedInfo(total, p.Page.Page, p.Page.PageSize), nil
	}

	sort.Slice(result, func(i, j int) bool {
		return result[i].ID().String() < result[j].ID().String()
	})

	return result, interfaces.NewPageBasedInfo(total, 1, int(total)), nil
}

func (r *Deployment) FindByProject(ctx context.Context, id id.ProjectID) (*deployment.Deployment, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	for _, d := range r.data {
		if d.Project() != nil && *d.Project() == id && d.IsHead() && r.f.CanRead(d.Workspace()) {
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

func (r *Deployment) FindByVersion(ctx context.Context, wsID id.WorkspaceID, projectID *id.ProjectID, version string) (*deployment.Deployment, error) {
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

func (r *Deployment) FindHead(ctx context.Context, wsID id.WorkspaceID, projectID *id.ProjectID) (*deployment.Deployment, error) {
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

func (r *Deployment) FindVersions(ctx context.Context, wsID id.WorkspaceID, projectID *id.ProjectID) ([]*deployment.Deployment, error) {
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
