package interactor

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
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
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachUser(ctx, mockUser)

	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
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
		},
		{
			name: "nonexistent workspace",
			param: interfaces.CreateProjectParam{
				WorkspaceID: wsid2,
			},
			wantErr: rerror.ErrNotFound,
		},
		// Once the operation check in the oss environment is completed, remove cooment out
		// {
		// 	name: "permission denied",
		// 	param: interfaces.CreateProjectParam{
		// 		WorkspaceID: ws.ID(),
		// 		Name:        lo.ToPtr("ccc"),
		// 		Description: lo.ToPtr("ddd"),
		// 		Archived:    lo.ToPtr(false),
		// 	},
		// 	permission: NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		// 		return false, nil
		// 	}),
		// 	wantErr: errors.New("permission denied"),
		// },
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.permission != nil {
				uc.permissionChecker = tt.permission
			}

			got, err := uc.Create(ctx, tt.param)

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
