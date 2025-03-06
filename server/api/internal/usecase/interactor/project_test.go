package interactor

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
)

func TestProject_Create(t *testing.T) {
	ctx := context.Background()

	uc := &Project{
		projectRepo:   memory.NewProject(),
		workspaceRepo: accountmemory.NewWorkspace(),
		transaction:   &usecasex.NopTransaction{},
	}

	ws := workspace.New().NewID().MustBuild()
	wsid2 := workspace.NewID()
	_ = uc.workspaceRepo.Save(ctx, ws)
	pId := project.NewID()
	defer project.MockNewID(pId)()

	// normal
	got, err := uc.Create(ctx, interfaces.CreateProjectParam{
		WorkspaceID: ws.ID(),
		Name:        lo.ToPtr("aaa"),
		Description: lo.ToPtr("bbb"),
		Archived:    lo.ToPtr(false),
	}, &usecase.Operator{
		AcOperator: &accountusecase.Operator{
			WritableWorkspaces: workspace.IDList{ws.ID()},
		},
	})

	assert.NoError(t, err)
	want := project.New().
		ID(pId).
		Workspace(ws.ID()).
		Name("aaa").
		Description("bbb").
		UpdatedAt(got.UpdatedAt()).
		MustBuild()
	assert.Equal(t, want, got)
	assert.Equal(t, want, lo.Must(uc.projectRepo.FindByID(ctx, pId)))

	// nonexistent workspace
	got, err = uc.Create(ctx, interfaces.CreateProjectParam{
		WorkspaceID: wsid2,
	}, &usecase.Operator{
		AcOperator: &accountusecase.Operator{
			WritableWorkspaces: workspace.IDList{wsid2},
		},
	})
	assert.Same(t, rerror.ErrNotFound, err)
	assert.Nil(t, got)

	// operation denied
	got, err = uc.Create(ctx, interfaces.CreateProjectParam{
		WorkspaceID: ws.ID(),
	}, &usecase.Operator{
		AcOperator: &accountusecase.Operator{
			ReadableWorkspaces: workspace.IDList{ws.ID()},
		},
	})
	assert.Same(t, interfaces.ErrOperationDenied, err)
	assert.Nil(t, got)
}
