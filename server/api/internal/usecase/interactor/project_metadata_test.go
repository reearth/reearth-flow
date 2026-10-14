package interactor

import (
	"context"
	"errors"
	"fmt"
	"testing"

	workspacemockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace/mockrepo"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.uber.org/mock/gomock"
)

func setupProjectMetadata(t *testing.T, checker *mockPermissionChecker) (*Project, *accountsworkspace.Workspace, context.Context, *accountsuser.User) {
	t.Helper()
	ws := factory.NewWorkspace(func(b *accountsworkspace.Builder) {})
	wsRepo := workspacemockrepo.NewMockWorkspaceRepo(gomock.NewController(t))
	wsRepo.EXPECT().FindByID(gomock.Any(), gomock.Any()).Return(ws, nil).AnyTimes()

	u := accountsuser.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()
	ctx := adapter.AttachUser(context.Background(), u)
	return setupProject(t, checker, wsRepo), ws, ctx, u
}

func allowAll() *mockPermissionChecker {
	return NewMockPermissionChecker(func(context.Context, string, string) (bool, error) { return true, nil })
}

func TestProject_Create_Metadata(t *testing.T) {
	uc, ws, ctx, _ := setupProjectMetadata(t, allowAll())

	got, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)
	assert.Equal(t, project.DefaultAlias(got.ID()), got.Alias())
	assert.Equal(t, project.VisibilityPublic, got.Visibility(), "new projects are public by default")

	got, err = uc.Create(ctx, interfaces.CreateProjectParam{
		WorkspaceID: ws.ID(),
		Alias:       lo.ToPtr("my-project"),
		Visibility:  lo.ToPtr(project.VisibilityPrivate),
		Readme:      lo.ToPtr("# Hello"),
		License:     lo.ToPtr("MIT"),
		Topics:      []string{"gis", " gis ", "plateau"},
	})
	require.NoError(t, err)
	assert.Equal(t, "my-project", got.Alias())
	assert.Equal(t, project.VisibilityPrivate, got.Visibility())
	assert.Equal(t, "# Hello", got.Readme())
	assert.Equal(t, "MIT", got.License())
	assert.Equal(t, []string{"gis", "plateau"}, got.Topics())
}

func TestProject_Create_RejectsBadMetadata(t *testing.T) {
	uc, ws, ctx, _ := setupProjectMetadata(t, allowAll())
	_, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID(), Alias: lo.ToPtr("my-project")})
	require.NoError(t, err)

	tests := []struct {
		err   error
		name  string
		param interfaces.CreateProjectParam
	}{
		{project.ErrInvalidAlias, "invalid alias", interfaces.CreateProjectParam{Alias: lo.ToPtr("a b")}},
		{project.ErrReservedAlias, "reserved alias", interfaces.CreateProjectParam{Alias: lo.ToPtr("p-mine")}},
		{interfaces.ErrProjectAliasAlreadyUsed, "taken alias", interfaces.CreateProjectParam{Alias: lo.ToPtr("my-project")}},
		{project.ErrInvalidVisibility, "invalid visibility", interfaces.CreateProjectParam{Visibility: lo.ToPtr(project.Visibility("unlisted"))}},
		{project.ErrTooManyTopics, "too many topics", interfaces.CreateProjectParam{Topics: make201Topics()}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tt.param.WorkspaceID = ws.ID()
			got, err := uc.Create(ctx, tt.param)
			assert.ErrorIs(t, err, tt.err)
			assert.Nil(t, got)
		})
	}
}

func make201Topics() []string {
	topics := make([]string, project.MaxTopics+1)
	for i := range topics {
		topics[i] = fmt.Sprintf("topic-%d", i)
	}
	return topics
}

func TestProject_Update_Metadata(t *testing.T) {
	uc, ws, ctx, _ := setupProjectMetadata(t, allowAll())
	p, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID(), Topics: []string{"gis"}, Readme: lo.ToPtr("keep")})
	require.NoError(t, err)
	_, err = uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID(), Alias: lo.ToPtr("taken-alias")})
	require.NoError(t, err)

	got, err := uc.Update(ctx, interfaces.UpdateProjectParam{
		ID:         p.ID(),
		Alias:      lo.ToPtr("renamed"),
		Visibility: lo.ToPtr(project.VisibilityPrivate),
		License:    lo.ToPtr("Apache-2.0"),
	})
	require.NoError(t, err)
	assert.Equal(t, "renamed", got.Alias())
	assert.Equal(t, project.VisibilityPrivate, got.Visibility())
	assert.Equal(t, "Apache-2.0", got.License())
	assert.Equal(t, "keep", got.Readme(), "a nil field is left as it is")
	assert.Equal(t, []string{"gis"}, got.Topics(), "nil topics are left as they are")

	got, err = uc.Update(ctx, interfaces.UpdateProjectParam{ID: p.ID(), Alias: lo.ToPtr("renamed")})
	require.NoError(t, err, "keeping its own alias is not a conflict")
	assert.Equal(t, "renamed", got.Alias())

	got, err = uc.Update(ctx, interfaces.UpdateProjectParam{ID: p.ID(), Topics: []string{}})
	require.NoError(t, err)
	assert.Empty(t, got.Topics(), "an empty list clears the topics")

	_, err = uc.Update(ctx, interfaces.UpdateProjectParam{ID: p.ID(), Alias: lo.ToPtr("taken-alias")})
	assert.ErrorIs(t, err, interfaces.ErrProjectAliasAlreadyUsed)
}

func TestProject_CheckAlias(t *testing.T) {
	uc, ws, ctx, _ := setupProjectMetadata(t, policyChecker("reader"))
	maintainerUC := &Project{projectRepo: uc.projectRepo, workspaceRepo: uc.workspaceRepo, transaction: uc.transaction, permissionChecker: allowAll()}
	p, err := maintainerUC.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID(), Alias: lo.ToPtr("taken-alias")})
	require.NoError(t, err)

	available, err := uc.CheckAlias(ctx, ws.ID(), "free-alias", nil)
	require.NoError(t, err)
	assert.True(t, available)

	available, err = uc.CheckAlias(ctx, ws.ID(), "taken-alias", nil)
	require.NoError(t, err)
	assert.False(t, available)

	available, err = uc.CheckAlias(ctx, ws.ID(), "taken-alias", lo.ToPtr(p.ID()))
	require.NoError(t, err)
	assert.True(t, available, "a project's own alias is available to it")

	available, err = uc.CheckAlias(ctx, accountsid.NewWorkspaceID(), "taken-alias", nil)
	require.NoError(t, err)
	assert.True(t, available, "aliases are per workspace")

	_, err = uc.CheckAlias(ctx, ws.ID(), "bad", nil)
	assert.ErrorIs(t, err, project.ErrInvalidAlias)

	_, err = uc.CheckAlias(ctx, ws.ID(), project.DefaultAlias(p.ID()), nil)
	assert.ErrorIs(t, err, project.ErrReservedAlias, "another project's default alias is reserved")

	denied := &Project{projectRepo: uc.projectRepo, permissionChecker: NewMockPermissionChecker(func(context.Context, string, string) (bool, error) { return false, nil })}
	_, err = denied.CheckAlias(ctx, ws.ID(), "free-alias", nil)
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
}

func TestProject_SetStar(t *testing.T) {
	uc, ws, ctx, u := setupProjectMetadata(t, policyChecker("reader"))
	maintainerUC := &Project{projectRepo: uc.projectRepo, workspaceRepo: uc.workspaceRepo, transaction: uc.transaction, permissionChecker: allowAll()}
	p, err := maintainerUC.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)
	uid := accountsid.UserID(u.ID())

	got, err := uc.SetStar(ctx, p.ID(), true)
	require.NoError(t, err, "a workspace reader can star")
	assert.True(t, got.IsStarredBy(uid))
	assert.Equal(t, int64(1), got.StarCount())

	got, err = uc.SetStar(ctx, p.ID(), true)
	require.NoError(t, err)
	assert.Equal(t, int64(1), got.StarCount(), "starring again is a no-op")

	got, err = uc.SetStar(ctx, p.ID(), false)
	require.NoError(t, err)
	assert.False(t, got.IsStarredBy(uid))
	assert.Zero(t, got.StarCount())

	_, err = uc.SetStar(context.Background(), p.ID(), true)
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied, "starring needs a user")

	denied := &Project{projectRepo: uc.projectRepo, permissionChecker: NewMockPermissionChecker(func(context.Context, string, string) (bool, error) { return false, nil })}
	_, err = denied.SetStar(ctx, p.ID(), true)
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
}

type fakePolicyChecker struct {
	err     error
	calls   int
	allowed bool
}

func (f *fakePolicyChecker) CheckPolicy(_ context.Context, req gateway.PolicyCheckRequest) (*gateway.PolicyCheckResponse, error) {
	f.calls++
	if f.err != nil {
		return nil, f.err
	}
	return &gateway.PolicyCheckResponse{Allowed: f.allowed, CheckType: req.CheckType, Value: req.Value}, nil
}

func TestProject_PrivateNeedsPlan(t *testing.T) {
	free := &fakePolicyChecker{allowed: false}
	uc, ws, ctx, _ := setupProjectMetadata(t, allowAll())
	uc.policyChecker = free

	got, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)
	assert.Equal(t, project.VisibilityPublic, got.Visibility())
	assert.Zero(t, free.calls, "a public project needs no policy call")

	_, err = uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID(), Visibility: lo.ToPtr(project.VisibilityPrivate)})
	assert.ErrorIs(t, err, interfaces.ErrPrivateProjectNotAllowed)

	_, err = uc.Update(ctx, interfaces.UpdateProjectParam{ID: got.ID(), Visibility: lo.ToPtr(project.VisibilityPrivate)})
	assert.ErrorIs(t, err, interfaces.ErrPrivateProjectNotAllowed)

	_, err = uc.Update(ctx, interfaces.UpdateProjectParam{ID: got.ID(), Visibility: lo.ToPtr(project.VisibilityPublic)})
	assert.NoError(t, err, "public is always allowed")

	can, err := uc.CanCreatePrivate(ctx, ws.ID())
	require.NoError(t, err)
	assert.False(t, can)
}

func TestProject_PrivateOnPaidPlan(t *testing.T) {
	paid := &fakePolicyChecker{allowed: true}
	uc, ws, ctx, _ := setupProjectMetadata(t, allowAll())
	uc.policyChecker = paid

	got, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID(), Visibility: lo.ToPtr(project.VisibilityPrivate)})
	require.NoError(t, err)
	assert.Equal(t, project.VisibilityPrivate, got.Visibility())

	can, err := uc.CanCreatePrivate(ctx, ws.ID())
	require.NoError(t, err)
	assert.True(t, can)

	// The team downgrades: the private project stays private and can still be edited.
	uc.policyChecker = &fakePolicyChecker{allowed: false}
	got, err = uc.Update(ctx, interfaces.UpdateProjectParam{ID: got.ID(), Name: lo.ToPtr("renamed"), Visibility: lo.ToPtr(project.VisibilityPrivate)})
	require.NoError(t, err, "re-sending the current visibility is not a change")
	assert.Equal(t, project.VisibilityPrivate, got.Visibility())
	assert.Equal(t, "renamed", got.Name())
}

func TestProject_PrivateWhenPolicyFails(t *testing.T) {
	uc, ws, ctx, _ := setupProjectMetadata(t, allowAll())
	uc.policyChecker = &fakePolicyChecker{err: errors.New("policy service down")}

	_, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID(), Visibility: lo.ToPtr(project.VisibilityPrivate)})
	assert.EqualError(t, err, "policy service down", "an unanswered check does not let a project go private")

	_, err = uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	assert.NoError(t, err, "public projects do not depend on the policy service")
}

func TestProject_CanCreatePrivate_NeedsMembership(t *testing.T) {
	uc, ws, ctx, _ := setupProjectMetadata(t, NewMockPermissionChecker(func(context.Context, string, string) (bool, error) { return false, nil }))
	_, err := uc.CanCreatePrivate(ctx, ws.ID())
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied)

	withoutChecker, ws2, ctx2, _ := setupProjectMetadata(t, allowAll())
	can, err := withoutChecker.CanCreatePrivate(ctx2, ws2.ID())
	require.NoError(t, err)
	assert.True(t, can, "without a policy checker every workspace may go private")
}
