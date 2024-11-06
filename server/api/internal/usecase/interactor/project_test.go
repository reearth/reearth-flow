package interactor

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
)

func setupProject(t *testing.T, permissionChecker *mockPermissionChecker) *Project {
	t.Helper()

	return &Project{
		projectRepo:       memory.NewProject(),
		workspaceRepo:     accountmemory.NewWorkspace(),
		transaction:       &usecasex.NopTransaction{},
		permissionChecker: permissionChecker,
	}
}

func TestProject_Create(t *testing.T) {
	ctx := context.Background()

	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, resource, action string) (bool, error) {
		return true, nil
	})

	uc := setupProject(t, mockPermissionCheckerTrue)

	ws := workspace.New().NewID().MustBuild()
	wsid2 := workspace.NewID()
	err := uc.workspaceRepo.Save(ctx, ws)
	assert.NoError(t, err, "workspace save should not return error")

	pId := project.NewID()
	defer project.MockNewID(pId)()

	tests := []struct {
		name       string
		param      interfaces.CreateProjectParam
		operator   *usecase.Operator
		permission *mockPermissionChecker
		wantErr    error
	}{
		{
			name: "normal creation",
			param: interfaces.CreateProjectParam{
				WorkspaceID: ws.ID(),
				Name:        lo.ToPtr("aaa"),
				Description: lo.ToPtr("bbb"),
				Archived:    lo.ToPtr(false),
			},
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					WritableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
		},
		{
			name: "nonexistent workspace",
			param: interfaces.CreateProjectParam{
				WorkspaceID: wsid2,
			},
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					WritableWorkspaces: workspace.IDList{wsid2},
				},
			},
			wantErr: rerror.ErrNotFound,
		},
		{
			name: "operation denied",
			param: interfaces.CreateProjectParam{
				WorkspaceID: ws.ID(),
			},
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					ReadableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			wantErr: interfaces.ErrOperationDenied,
		},
		{
			name: "permission denied",
			param: interfaces.CreateProjectParam{
				WorkspaceID: ws.ID(),
				Name:        lo.ToPtr("ccc"),
				Description: lo.ToPtr("ddd"),
				Archived:    lo.ToPtr(false),
			},
			operator: &usecase.Operator{
				AcOperator: &accountusecase.Operator{
					WritableWorkspaces: workspace.IDList{ws.ID()},
				},
			},
			permission: NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, resource, action string) (bool, error) {
				return false, nil
			}),
			wantErr: errors.New("permission denied"),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.permission != nil {
				uc.permissionChecker = tt.permission
			}

			got, err := uc.Create(ctx, tt.param, tt.operator)

			if tt.wantErr != nil {
				assert.EqualError(t, err, tt.wantErr.Error())
				assert.Nil(t, got, "project should be nil when error is expected")
				return
			}
			assert.NoError(t, err)
			want := project.New().
				ID(pId).
				Workspace(ws.ID()).
				Name(*tt.param.Name).
				Description(*tt.param.Description).
				UpdatedAt(got.UpdatedAt()).
				MustBuild()
			assert.Equal(t, want, got)
		})
	}
}
