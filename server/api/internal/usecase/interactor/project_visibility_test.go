package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// memberOf allows everything in one workspace and nothing elsewhere.
type memberOf accountsid.WorkspaceID

func (m memberOf) CheckPermission(_ context.Context, _, _ string, ws ...accountsid.WorkspaceID) (bool, error) {
	return len(ws) == 1 && ws[0] == accountsid.WorkspaceID(m), nil
}

type visibilityFixture struct {
	ctx           context.Context
	uc            *Project
	user          *accountsuser.User
	minePrivate   *project.Project
	theirPublic   *project.Project
	theirPrivate  *project.Project
	theirArchived *project.Project
	mine, theirs  accountsid.WorkspaceID
}

// setupVisibility saves projects in a workspace the caller belongs to and in
// one they do not.
func setupVisibility(t *testing.T) visibilityFixture {
	t.Helper()
	f := visibilityFixture{mine: accountsid.NewWorkspaceID(), theirs: accountsid.NewWorkspaceID()}
	repo := memory.NewProject()
	f.uc = &Project{
		projectRepo:       repo,
		transaction:       usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
		permissionChecker: memberOf(f.mine),
	}
	f.user = accountsuser.New().NewID().Name("u").Email("u@example.com").MustBuild()
	f.ctx = adapter.AttachUser(context.Background(), f.user)

	save := func(ws accountsid.WorkspaceID, name, alias string, v project.Visibility, archived bool) *project.Project {
		p := project.New().NewID().Workspace(ws).Name(name).Alias(alias).Visibility(v).IsArchived(archived).MustBuild()
		require.NoError(t, repo.Save(context.Background(), p))
		return p
	}
	f.minePrivate = save(f.mine, "mine private", "mine-private", project.VisibilityPrivate, false)
	save(f.mine, "mine public", "mine-public", project.VisibilityPublic, false)
	f.theirPublic = save(f.theirs, "their public", "their-public", project.VisibilityPublic, false)
	f.theirPrivate = save(f.theirs, "their private", "their-private", project.VisibilityPrivate, false)
	f.theirArchived = save(f.theirs, "their archived", "their-archived", project.VisibilityPublic, true)
	return f
}

func names(ps []*project.Project) []string {
	return lo.Map(ps, func(p *project.Project, _ int) string { return p.Name() })
}

func TestProject_FindViewable(t *testing.T) {
	f := setupVisibility(t)

	got, err := f.uc.FindViewable(f.ctx, f.minePrivate.ID())
	require.NoError(t, err)
	assert.Equal(t, f.minePrivate.ID(), got.ID(), "members see private projects")

	got, err = f.uc.FindViewable(f.ctx, f.theirPublic.ID())
	require.NoError(t, err)
	assert.Equal(t, f.theirPublic.ID(), got.ID(), "anyone sees a public project")

	_, err = f.uc.FindViewable(f.ctx, f.theirPrivate.ID())
	assert.ErrorIs(t, err, rerror.ErrNotFound, "a hidden project looks missing")

	_, err = f.uc.FindViewable(f.ctx, f.theirArchived.ID())
	assert.ErrorIs(t, err, rerror.ErrNotFound, "archiving hides a public project from non-members")

	_, err = f.uc.FindViewable(context.Background(), f.theirPublic.ID())
	assert.NoError(t, err, "no user is needed for a public project")

	got, err = f.uc.FindViewableByAlias(f.ctx, f.theirs, "their-public")
	require.NoError(t, err)
	assert.Equal(t, f.theirPublic.ID(), got.ID())

	_, err = f.uc.FindViewableByAlias(f.ctx, f.theirs, "their-private")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestProject_Search(t *testing.T) {
	f := setupVisibility(t)
	sortByName := interfaces.ProjectSearchParam{SortBy: interfaces.ProjectSortName, SortAscending: true}

	t.Run("member", func(t *testing.T) {
		p := sortByName
		p.Workspace = &f.mine
		got, info, err := f.uc.Search(f.ctx, p)
		require.NoError(t, err)
		assert.Equal(t, []string{"mine private", "mine public"}, names(got))
		assert.Equal(t, int64(2), info.TotalCount)
	})

	t.Run("non-member gets public, unarchived projects", func(t *testing.T) {
		p := sortByName
		p.Workspace = &f.theirs
		p.IncludeArchived = true
		got, _, err := f.uc.Search(f.ctx, p)
		require.NoError(t, err)
		assert.Equal(t, []string{"their public"}, names(got))
	})

	t.Run("non-member asking for private gets nothing", func(t *testing.T) {
		p := sortByName
		p.Workspace = &f.theirs
		p.Visibility = lo.ToPtr(project.VisibilityPrivate)
		got, info, err := f.uc.Search(f.ctx, p)
		require.NoError(t, err)
		assert.Empty(t, got)
		assert.Zero(t, info.TotalCount)
	})

	t.Run("anonymous", func(t *testing.T) {
		p := sortByName
		p.Workspace = &f.mine
		got, _, err := f.uc.Search(context.Background(), p)
		require.NoError(t, err)
		assert.Equal(t, []string{"mine public"}, names(got))
	})

	t.Run("every workspace", func(t *testing.T) {
		got, _, err := f.uc.Search(f.ctx, sortByName)
		require.NoError(t, err)
		assert.Equal(t, []string{"mine public", "their public"}, names(got), "only public projects, even the caller's own")
	})

	t.Run("pages", func(t *testing.T) {
		p := sortByName
		p.PageSize = 1
		p.Page = 2
		got, info, err := f.uc.Search(f.ctx, p)
		require.NoError(t, err)
		assert.Equal(t, []string{"their public"}, names(got))
		assert.Equal(t, int64(2), info.TotalCount)
		assert.Equal(t, 2, info.CurrentPage)
	})
}

func TestProject_SetStar_Public(t *testing.T) {
	f := setupVisibility(t)

	got, err := f.uc.SetStar(f.ctx, f.theirPublic.ID(), true)
	require.NoError(t, err)
	assert.True(t, got.IsStarredBy(accountsid.UserID(f.user.ID())), "anyone may star a public project")

	_, err = f.uc.SetStar(f.ctx, f.theirPrivate.ID(), true)
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied)

	_, err = f.uc.SetStar(f.ctx, f.theirArchived.ID(), true)
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied, "an archived project is not open to non-members")

	_, err = f.uc.SetStar(context.Background(), f.theirPublic.ID(), true)
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied, "starring needs a user")
}

// memberOfAny allows everything in the given workspaces and nothing elsewhere.
type memberOfAny accountsid.WorkspaceIDList

func (m memberOfAny) CheckPermission(_ context.Context, _, _ string, ws ...accountsid.WorkspaceID) (bool, error) {
	return len(ws) == 1 && accountsid.WorkspaceIDList(m).Has(ws[0]), nil
}

// failingChecker fails every permission check.
type failingChecker struct{}

func (failingChecker) CheckPermission(context.Context, string, string, ...accountsid.WorkspaceID) (bool, error) {
	return false, errors.New("permission service down")
}

func TestProject_Search_Workspaces(t *testing.T) {
	t.Parallel()

	type fixture struct {
		ctx            context.Context
		uc             *Project
		one, two, them accountsid.WorkspaceID
	}
	setup := func(t *testing.T) fixture {
		t.Helper()
		f := fixture{one: accountsid.NewWorkspaceID(), two: accountsid.NewWorkspaceID(), them: accountsid.NewWorkspaceID()}
		repo := memory.NewProject()
		f.uc = &Project{
			projectRepo:       repo,
			transaction:       usecasex.NewTransactor(&usecasex.NopTransaction{}, 0),
			permissionChecker: memberOfAny{f.one, f.two},
		}
		u := accountsuser.New().NewID().Name("u").Email("u@example.com").MustBuild()
		f.ctx = adapter.AttachUser(context.Background(), u)

		base := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
		save := func(ws accountsid.WorkspaceID, name string, v project.Visibility, archived bool, hour int) {
			p := project.New().NewID().Workspace(ws).Name(name).Visibility(v).IsArchived(archived).
				UpdatedAt(base.Add(time.Duration(hour) * time.Hour)).MustBuild()
			require.NoError(t, repo.Save(context.Background(), p))
		}
		save(f.one, "one private", project.VisibilityPrivate, false, 0)
		save(f.two, "two public", project.VisibilityPublic, false, 1)
		save(f.one, "one archived", project.VisibilityPublic, true, 2)
		save(f.them, "their public", project.VisibilityPublic, false, 3)
		save(f.them, "their private", project.VisibilityPrivate, false, 4)
		return f
	}

	t.Run("lists member workspaces as one page and skips the rest", func(t *testing.T) {
		t.Parallel()
		f := setup(t)

		got, info, err := f.uc.Search(f.ctx, interfaces.ProjectSearchParam{
			Workspaces: accountsid.WorkspaceIDList{f.one, f.them, f.two, f.one},
		})
		require.NoError(t, err)
		assert.Equal(t, []string{"two public", "one private"}, names(got), "newest first; non-member and archived projects left out")
		assert.Equal(t, int64(2), info.TotalCount)
	})

	t.Run("members may include archived and filter by visibility", func(t *testing.T) {
		t.Parallel()
		f := setup(t)

		got, _, err := f.uc.Search(f.ctx, interfaces.ProjectSearchParam{
			Workspaces:      accountsid.WorkspaceIDList{f.one, f.two, f.them},
			IncludeArchived: true,
			Visibility:      lo.ToPtr(project.VisibilityPublic),
		})
		require.NoError(t, err)
		assert.Equal(t, []string{"one archived", "two public"}, names(got))
	})

	t.Run("pages across workspaces", func(t *testing.T) {
		t.Parallel()
		f := setup(t)

		got, info, err := f.uc.Search(f.ctx, interfaces.ProjectSearchParam{
			Workspaces: accountsid.WorkspaceIDList{f.one, f.two},
			Page:       2,
			PageSize:   1,
		})
		require.NoError(t, err)
		assert.Equal(t, []string{"one private"}, names(got))
		assert.Equal(t, int64(2), info.TotalCount)
	})

	t.Run("no member workspace gives an empty page", func(t *testing.T) {
		t.Parallel()
		f := setup(t)

		got, info, err := f.uc.Search(f.ctx, interfaces.ProjectSearchParam{Workspaces: accountsid.WorkspaceIDList{f.them}})
		require.NoError(t, err)
		assert.Empty(t, got)
		assert.Zero(t, info.TotalCount)
	})

	t.Run("anonymous callers are members of nothing", func(t *testing.T) {
		t.Parallel()
		f := setup(t)

		got, info, err := f.uc.Search(context.Background(), interfaces.ProjectSearchParam{Workspaces: accountsid.WorkspaceIDList{f.one, f.two}})
		require.NoError(t, err)
		assert.Empty(t, got)
		assert.Zero(t, info.TotalCount)
	})

	t.Run("a failing permission check is returned", func(t *testing.T) {
		t.Parallel()
		f := setup(t)
		f.uc.permissionChecker = failingChecker{}

		_, _, err := f.uc.Search(f.ctx, interfaces.ProjectSearchParam{Workspaces: accountsid.WorkspaceIDList{f.one}})
		assert.Error(t, err)
	})
}
