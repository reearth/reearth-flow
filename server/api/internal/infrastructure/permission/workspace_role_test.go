package permission

import (
	"context"
	"errors"
	"testing"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"

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

// The guard has three deliberate fall-throughs where it cannot determine the
// caller's role and keeps the Cerbos verdict rather than denying a real user.
// They are pinned so switching any of them to fail-closed is a conscious
// change rather than an accident, since each one is a hole while it lasts.

// sequencedWorkspaceRepo returns a different result per FindByID call, so a
// test can let the alias lookup succeed and fail the guard's membership
// lookup that follows it.
type sequencedWorkspaceRepo struct {
	gqlworkspace.WorkspaceRepo
	results []struct {
		ws  *workspace.Workspace
		err error
	}
	calls int
}

func (r *sequencedWorkspaceRepo) FindByID(_ context.Context, _ string) (*workspace.Workspace, error) {
	i := r.calls
	r.calls++
	if i >= len(r.results) {
		i = len(r.results) - 1
	}
	return r.results[i].ws, r.results[i].err
}

// TestChecker_MembershipLookupErrorKeepsCerbosVerdict pins the guard's
// lookup-error fall-through. The alias lookup must succeed first, otherwise
// resolveAlias fails closed and the guard is never reached.
func TestChecker_MembershipLookupErrorKeepsCerbosVerdict(t *testing.T) {
	ws, ctx, _ := wsWithMember(t, role.RoleReader)
	repo := &sequencedWorkspaceRepo{results: []struct {
		ws  *workspace.Workspace
		err error
	}{
		{ws: ws}, // resolveAlias
		{err: errors.New("accounts unavailable")}, // the guard's membership lookup
	}}
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, repo, "flow")

	allowed, err := c.CheckPermission(ctx, "deployment", "any", ws.ID())

	require.NoError(t, err)
	assert.True(t, allowed, "an unavailable membership lookup must not deny a real user")
	assert.Equal(t, 2, repo.calls, "the guard must make its own membership lookup after the alias resolves")
}

// TestChecker_NilWorkspaceKeepsCerbosVerdict: FindByID may return (nil, nil).
// Reading Members() off that would panic.
func TestChecker_NilWorkspaceKeepsCerbosVerdict(t *testing.T) {
	ws, ctx, _ := wsWithMember(t, role.RoleReader)
	repo := &sequencedWorkspaceRepo{results: []struct {
		ws  *workspace.Workspace
		err error
	}{
		{ws: ws},  // resolveAlias
		{ws: nil}, // the guard's membership lookup finds nothing
	}}
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, repo, "flow")

	require.NotPanics(t, func() {
		allowed, err := c.CheckPermission(ctx, "deployment", "any", ws.ID())
		require.NoError(t, err)
		assert.True(t, allowed)
	})
}

// An empty membership list means accounts returned no members at all, which
// would otherwise deny every caller for that workspace.
func TestChecker_EmptyMembershipKeepsCerbosVerdict(t *testing.T) {
	_, ctx, _ := wsWithMember(t, role.RoleReader)
	noMembers := workspace.New().NewID().Alias("acme").MustBuild()
	cer := &fakeCerbosRepo{result: &cerbos.CheckPermissionResult{Allowed: true}}
	c := NewChecker(cer, &fakeWorkspaceRepo{ws: noMembers}, "flow")

	allowed, err := c.CheckPermission(ctx, "deployment", "any", noMembers.ID())

	require.NoError(t, err)
	assert.True(t, allowed, "with no membership data the guard must not deny a real user")
}
