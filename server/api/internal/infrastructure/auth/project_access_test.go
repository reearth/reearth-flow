package auth

import (
	"context"
	"errors"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type fakeUsers struct {
	u   *accountsuser.User
	err error
}

func (f fakeUsers) FindMe(context.Context) (*accountsuser.User, error) { return f.u, f.err }

// fakeChecker grants the listed actions on the project resource in one
// workspace and records whether a user was attached to the context it was
// called with. Any other resource is denied, as Cerbos denies resources absent
// from its policy store (projectDocument is not published there).
type fakeChecker struct {
	allow     map[string]bool
	workspace accountsid.WorkspaceID
	sawUser   bool
}

func (c *fakeChecker) CheckPermission(ctx context.Context, resource, action string, ws ...accountsid.WorkspaceID) (bool, error) {
	c.sawUser = adapter.User(ctx) != nil
	if resource != rbac.ResourceProject || len(ws) != 1 || ws[0] != c.workspace {
		return false, nil
	}
	return c.allow[action], nil
}

func newAccessFixture(t *testing.T) (*ProjectDocumentAccess, *project.Project, *fakeChecker) {
	t.Helper()
	ws := accountsid.NewWorkspaceID()
	p := project.New().NewID().Workspace(ws).Name("p").MustBuild()
	projects := memory.NewProject()
	require.NoError(t, projects.Save(context.Background(), p))

	checker := &fakeChecker{workspace: ws, allow: map[string]bool{}}
	return &ProjectDocumentAccess{
		Projects: projects,
		Access:   memory.NewProjectAccess(),
		Users:    fakeUsers{u: accountsuser.New().NewID().Name("u").Email("u@example.com").MustBuild()},
		Checker:  checker,
	}, p, checker
}

func TestUserAccess_ReaderIsReadOnly(t *testing.T) {
	a, p, checker := newAccessFixture(t)
	checker.allow[rbac.ActionRead] = true

	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.True(t, acc.Read)
	assert.False(t, acc.Edit, "a workspace reader must not get write access to the document")
	assert.True(t, checker.sawUser, "the user must be attached so the workspace-role guard runs")
}

func TestUserAccess_WriterCanEdit(t *testing.T) {
	a, p, checker := newAccessFixture(t)
	checker.allow[rbac.ActionRead] = true
	// project:any is writer-or-higher; project:edit is maintainer-only and must
	// not be what gates live editing.
	checker.allow[rbac.ActionAny] = true

	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.True(t, acc.Read)
	assert.True(t, acc.Edit)
}

func TestUserAccess_NamesTheUser(t *testing.T) {
	t.Parallel()

	a, p, checker := newAccessFixture(t)
	checker.allow[rbac.ActionRead] = true
	me, err := a.Users.FindMe(context.Background())
	require.NoError(t, err)

	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.Equal(t, me.ID().String(), acc.UserID)
}

func TestUserAccess_NonMemberDenied(t *testing.T) {
	a, p, _ := newAccessFixture(t)

	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.False(t, acc.Read)
	assert.False(t, acc.Edit)
}

func TestUserAccess_UnknownOrMalformedProjectDenied(t *testing.T) {
	a, _, checker := newAccessFixture(t)
	checker.allow[rbac.ActionRead] = true
	checker.allow[rbac.ActionAny] = true

	for _, pid := range []string{id.NewProjectID().String(), "not-a-project-id", ""} {
		acc, err := a.UserAccess(context.Background(), pid)
		require.NoError(t, err, pid)
		assert.False(t, acc.Read, pid)
	}
}

func TestUserAccess_UnresolvableUserDenied(t *testing.T) {
	a, p, checker := newAccessFixture(t)
	checker.allow[rbac.ActionRead] = true

	a.Users = fakeUsers{}
	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.False(t, acc.Read)

	a.Users = fakeUsers{err: errors.New("accounts down")}
	acc, err = a.UserAccess(context.Background(), p.ID().String())
	assert.Error(t, err)
	assert.False(t, acc.Read)
}

func TestUserAccess_SkipPermissionCheckStillNeedsUserAndProject(t *testing.T) {
	a, p, _ := newAccessFixture(t)
	a.SkipPermissionCheck = true

	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.True(t, acc.Read)
	assert.True(t, acc.Edit)

	acc, err = a.UserAccess(context.Background(), id.NewProjectID().String())
	require.NoError(t, err)
	assert.False(t, acc.Read)
}

func lockProject(t *testing.T, a *ProjectDocumentAccess, p *project.Project) {
	t.Helper()
	p.SetIsLocked(true)
	require.NoError(t, a.Projects.Save(context.Background(), p))
}

func TestUserAccess_LockedProjectIsReadOnlyForWriters(t *testing.T) {
	t.Parallel()

	a, p, checker := newAccessFixture(t)
	checker.allow[rbac.ActionRead] = true
	checker.allow[rbac.ActionAny] = true
	lockProject(t, a, p)

	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.True(t, acc.Read, "locking must not take away read access")
	assert.False(t, acc.Edit, "nobody may edit a locked project's document")
}

func TestUserAccess_LockedProjectIsReadOnlyWithoutPermissionCheck(t *testing.T) {
	t.Parallel()

	a, p, _ := newAccessFixture(t)
	a.SkipPermissionCheck = true
	lockProject(t, a, p)

	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.True(t, acc.Read)
	assert.False(t, acc.Edit)
}

func TestUserAccess_LockedProjectStillDeniesNonMembers(t *testing.T) {
	t.Parallel()

	a, p, _ := newAccessFixture(t)
	lockProject(t, a, p)

	acc, err := a.UserAccess(context.Background(), p.ID().String())
	require.NoError(t, err)
	assert.False(t, acc.Read)
	assert.False(t, acc.Edit)
}

func saveAccess(t *testing.T, a *ProjectDocumentAccess, pid id.ProjectID, public bool) string {
	t.Helper()
	pa := projectAccess.New().NewID().Project(pid).IsPublic(public).Token("share-" + pid.String()).MustBuild()
	require.NoError(t, a.Access.Save(context.Background(), pa))
	return pa.Token()
}

func TestSharedAccess(t *testing.T) {
	a, p, _ := newAccessFixture(t)
	token := saveAccess(t, a, p.ID(), true)

	ok, err := a.SharedAccess(context.Background(), token, p.ID().String())
	require.NoError(t, err)
	assert.True(t, ok)

	ok, err = a.SharedAccess(context.Background(), token, id.NewProjectID().String())
	require.NoError(t, err)
	assert.False(t, ok, "a share token must only open its own project")

	ok, err = a.SharedAccess(context.Background(), "unknown", p.ID().String())
	require.NoError(t, err)
	assert.False(t, ok)
}

func TestSharedAccess_UnsharedProjectDenied(t *testing.T) {
	a, p, _ := newAccessFixture(t)
	token := saveAccess(t, a, p.ID(), false)

	ok, err := a.SharedAccess(context.Background(), token, p.ID().String())
	require.NoError(t, err)
	assert.False(t, ok, "a project whose sharing was turned off must not open")
}
