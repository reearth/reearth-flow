package interactor

import (
	"context"
	"testing"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	workspacemockrepo "github.com/reearth/reearth-flow/api/pkg/workspace/mockrepo"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
)

func setupProject(t *testing.T, permissionChecker *mockPermissionChecker, workspaceRepo gqlworkspace.WorkspaceRepo) *Project {
	t.Helper()

	return &Project{
		projectRepo:       memory.NewProject(),
		workspaceRepo:     workspaceRepo,
		transaction:       &usecasex.NopTransaction{},
		permissionChecker: permissionChecker,
	}
}

func TestProject_Create(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	ctx := context.Background()

	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})

	ws := factory.NewWorkspace(func(b *accountsworkspace.Builder) {})
	wsid2 := accountsworkspace.NewID()

	pId := project.NewID()
	defer project.MockNewID(pId)()

	tests := []struct {
		name              string
		param             interfaces.CreateProjectParam
		mockFindWorkspace func(m *workspacemockrepo.MockWorkspaceRepo)
		permission        *mockPermissionChecker
		wantErr           error
	}{
		{
			name: "normal creation",
			param: interfaces.CreateProjectParam{
				WorkspaceID: ws.ID(),
				Name:        lo.ToPtr("aaa"),
				Description: lo.ToPtr("bbb"),
				Archived:    lo.ToPtr(false),
			},
			mockFindWorkspace: func(m *workspacemockrepo.MockWorkspaceRepo) {
				m.EXPECT().FindByID(gomock.Any(), gomock.Any()).Return(ws, nil)
			},
		},
		{
			name: "nonexistent workspace",
			param: interfaces.CreateProjectParam{
				WorkspaceID: wsid2,
			},
			wantErr: rerror.ErrNotFound,
			mockFindWorkspace: func(m *workspacemockrepo.MockWorkspaceRepo) {
				m.EXPECT().FindByID(gomock.Any(), gomock.Any()).Return(ws, rerror.ErrNotFound)
			},
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
			mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
			if tt.mockFindWorkspace != nil {
				tt.mockFindWorkspace(mockWorkspaceRepo)
			}

			uc := setupProject(t, mockPermissionCheckerTrue, mockWorkspaceRepo)

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
