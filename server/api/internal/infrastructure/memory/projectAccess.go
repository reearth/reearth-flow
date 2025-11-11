package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/reearth/reearthx/rerror"
)

type ProjectAccess struct {
	data map[id.ProjectAccessID]*projectAccess.ProjectAccess
	lock sync.Mutex
}

func NewProjectAccess() repo.ProjectAccess {
	return &ProjectAccess{
		data: map[id.ProjectAccessID]*projectAccess.ProjectAccess{},
	}
}

func NewProjectAccessWith(items ...*projectAccess.ProjectAccess) repo.ProjectAccess {
	pa := NewProjectAccess()
	ctx := context.Background()
	for _, i := range items {
		_ = pa.Save(ctx, i)
	}
	return pa
}

func (pa *ProjectAccess) FindByProjectID(ctx context.Context, projectId id.ProjectID) (*projectAccess.ProjectAccess, error) {
	pa.lock.Lock()
	defer pa.lock.Unlock()

	for _, access := range pa.data {
		if access.Project() == projectId {
			return access, nil
		}
	}
	return nil, rerror.ErrNotFound
}

func (pa *ProjectAccess) FindByToken(ctx context.Context, token string) (*projectAccess.ProjectAccess, error) {
	pa.lock.Lock()
	defer pa.lock.Unlock()

	for _, access := range pa.data {
		if access.Token() == token {
			return access, nil
		}
	}
	return nil, rerror.ErrNotFound
}

func (pa *ProjectAccess) Save(ctx context.Context, projectAccess *projectAccess.ProjectAccess) error {
	pa.lock.Lock()
	defer pa.lock.Unlock()

	pa.data[projectAccess.ID()] = projectAccess
	return nil
}
