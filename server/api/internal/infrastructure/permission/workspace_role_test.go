package permission

import (
	"context"
	"testing"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient/cerbos"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-accounts/server/pkg/role"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// wsWithMember builds a workspace where u holds r, and a context carrying u.
func wsWithMember(t *testing.T, r role.RoleType) (*workspace.Workspace, context.Context, accountsid.UserID) {
	t.Helper()
	u := accountsuser.New().NewID().Name("u").Email("u@example.com").MustBuild()
	uid := accountsid.UserID(u.ID())
	ws := workspace.New().NewID().Alias("acme").
		Members(map[workspace.UserID]workspace.Member{uid: {Role: r}}).
		MustBuild()
	return ws, adapter.AttachUser(context.Background(), u), uid
}

// TestChecker_ReaderDeniedEvenWhenCerbosAllows is the regression this guard
// exists for: accounts unions each user's stale global roles into every check,
// so a workspace reader can arrive at Cerbos carrying maintainer and be
// allowed. The workspace role must still decide.
func TestChecker_ReaderDeniedEvenWhenCerbosAllows(t *testing.T) {
	ws, ctx, _ := wsWithMember(t, role.RoleReader)
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, &fakeWorkspaceRepo{ws: ws}, "flow")

	allowed, err := c.CheckPermission(ctx, "deployment", "any", ws.ID())

	require.NoError(t, err)
	assert.False(t, allowed, "a workspace reader must not get deployment/any, whatever Cerbos said")
}

func TestChecker_ReaderStillAllowedOnReadOnlyActions(t *testing.T) {
	ws, ctx, _ := wsWithMember(t, role.RoleReader)
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, &fakeWorkspaceRepo{ws: ws}, "flow")

	allowed, err := c.CheckPermission(ctx, "project", "read", ws.ID())

	require.NoError(t, err)
	assert.True(t, allowed, "readers must keep the access the policy grants them")
}

func TestChecker_WriterAllowedOnContentActions(t *testing.T) {
	ws, ctx, _ := wsWithMember(t, role.RoleWriter)
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, &fakeWorkspaceRepo{ws: ws}, "flow")

	allowed, err := c.CheckPermission(ctx, "deployment", "any", ws.ID())

	require.NoError(t, err)
	assert.True(t, allowed)
}

func TestChecker_NonMemberDenied(t *testing.T) {
	_, ctx, _ := wsWithMember(t, role.RoleOwner)
	other := workspace.New().NewID().Alias("other").
		Members(map[workspace.UserID]workspace.Member{accountsid.NewUserID(): {Role: role.RoleOwner}}).
		MustBuild()
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, &fakeWorkspaceRepo{ws: other}, "flow")

	allowed, err := c.CheckPermission(ctx, "deployment", "any", other.ID())

	require.NoError(t, err)
	assert.False(t, allowed, "a non-member must be denied even if Cerbos allowed")
}

// TestChecker_NoUserPrincipalKeepsCerbosVerdict: API triggers and integrations
// run without a user and are not workspace members, so the guard must not
// deny them.
func TestChecker_NoUserPrincipalKeepsCerbosVerdict(t *testing.T) {
	ws, _, _ := wsWithMember(t, role.RoleReader)
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, &fakeWorkspaceRepo{ws: ws}, "flow")

	allowed, err := c.CheckPermission(context.Background(), "deployment", "any", ws.ID())

	require.NoError(t, err)
	assert.True(t, allowed)
}

func TestChecker_DeniedByCerbosStaysDenied(t *testing.T) {
	ws, ctx, _ := wsWithMember(t, role.RoleOwner)
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: false}}
	c := NewChecker(cer, &fakeWorkspaceRepo{ws: ws}, "flow")

	allowed, err := c.CheckPermission(ctx, "deployment", "any", ws.ID())

	require.NoError(t, err)
	assert.False(t, allowed, "the guard can only deny; it must never turn a Cerbos deny into an allow")
}
