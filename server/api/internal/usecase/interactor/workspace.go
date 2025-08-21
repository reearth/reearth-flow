package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workspace"
)

type Workspace struct {
	workspaceRepo workspace.Repo
}

func NewWorkspace(workspaceRepo workspace.Repo) interfaces.Workspace {
	return &Workspace{
		workspaceRepo: workspaceRepo,
	}
}

func (i *Workspace) FindByIDs(ctx context.Context, ids id.WorkspaceIDList) (workspace.WorkspaceList, error) {
	return i.workspaceRepo.FindByIDs(ctx, ids)
}
