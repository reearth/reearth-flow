package postgres_test

import (
	"context"
	"io/fs"
	"sort"
	"strings"
	"sync"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	flowdb "github.com/reearth/reearth-flow/db"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/pgxx/pgxtest"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestProject_SaveFindMetadata(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	u := accountsid.NewUserID()
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).
		Alias("my-project").Visibility(project.VisibilityPublic).
		Readme("# Hello").License("MIT").Topics([]string{"gis", "plateau"}).
		StarredBy([]string{u.String()}).
		UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, p))

	got, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.Equal(t, "my-project", got.Alias())
	assert.Equal(t, project.VisibilityPublic, got.Visibility())
	assert.Equal(t, "# Hello", got.Readme())
	assert.Equal(t, "MIT", got.License())
	assert.Equal(t, []string{"gis", "plateau"}, got.Topics())
	assert.True(t, got.IsStarredBy(u), "the first save writes the stars it is given, so a copied project keeps them")
}

func TestProject_Save_DuplicateAlias(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	require.NoError(t, r.Save(ctx, project.New().NewID().Workspace(wid).Alias("taken").UpdatedAt(time.Now()).MustBuild()))

	err := r.Save(ctx, project.New().NewID().Workspace(wid).Alias("taken").UpdatedAt(time.Now()).MustBuild())
	assert.ErrorIs(t, err, interfaces.ErrProjectAliasAlreadyUsed)

	other := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).Alias("taken").UpdatedAt(time.Now()).MustBuild()
	assert.NoError(t, r.Save(ctx, other), "aliases are per workspace")
}

func TestProject_FindByAlias(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	other := accountsid.NewWorkspaceID()
	p := project.New().NewID().Workspace(wid).Alias("my-project").UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, p))

	got, err := r.FindByAlias(ctx, wid, "my-project")
	require.NoError(t, err)
	assert.Equal(t, p.ID(), got.ID())

	_, err = r.FindByAlias(ctx, other, "my-project")
	assert.ErrorIs(t, err, rerror.ErrNotFound)

	_, err = r.FindByAlias(ctx, wid, "missing")
	assert.ErrorIs(t, err, rerror.ErrNotFound)

	filtered := r.Filtered(repo.WorkspaceFilter{Readable: accountsid.WorkspaceIDList{other}})
	_, err = filtered.FindByAlias(ctx, wid, "my-project")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestProject_SetStar(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	p := newProject(wid, id.NewProjectID(), "p")
	require.NoError(t, r.Save(ctx, p))
	u1, u2 := accountsid.NewUserID(), accountsid.NewUserID()

	require.NoError(t, r.SetStar(ctx, p.ID(), u1, true))
	require.NoError(t, r.SetStar(ctx, p.ID(), u1, true))
	require.NoError(t, r.SetStar(ctx, p.ID(), u2, true))
	got, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.Equal(t, int64(2), got.StarCount(), "starring twice counts once")

	require.NoError(t, r.SetStar(ctx, p.ID(), u1, false))
	require.NoError(t, r.SetStar(ctx, p.ID(), u1, false))
	got, err = r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.False(t, got.IsStarredBy(u1))
	assert.True(t, got.IsStarredBy(u2))

	assert.ErrorIs(t, r.SetStar(ctx, id.NewProjectID(), u1, true), rerror.ErrNotFound)

	filtered := r.Filtered(repo.WorkspaceFilter{Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()}})
	assert.ErrorIs(t, filtered.SetStar(ctx, p.ID(), u1, true), rerror.ErrNotFound)
}

func TestProject_SetStar_Concurrent(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	p := newProject(accountsid.NewWorkspaceID(), id.NewProjectID(), "p")
	require.NoError(t, r.Save(ctx, p))

	const n = 20
	var wg sync.WaitGroup
	for range n {
		wg.Add(1)
		go func() {
			defer wg.Done()
			assert.NoError(t, r.SetStar(ctx, p.ID(), accountsid.NewUserID(), true))
		}()
	}
	wg.Wait()

	got, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.Equal(t, int64(n), got.StarCount())
}

func TestProject_SaveKeepsStarsAddedSinceRead(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	p := newProject(accountsid.NewWorkspaceID(), id.NewProjectID(), "before")
	require.NoError(t, r.Save(ctx, p))

	stale, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	u := accountsid.NewUserID()
	require.NoError(t, r.SetStar(ctx, p.ID(), u, true))

	stale.SetUpdateName("after")
	require.NoError(t, r.Save(ctx, stale))

	got, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.Equal(t, "after", got.Name())
	assert.True(t, got.IsStarredBy(u), "saving a stale read must not drop the star")
}

// TestMigration_AddProjectMetadata_ExistingRows applies the project-metadata
// migration to a database that already holds projects, which a fresh test
// database never exercises.
func TestMigration_AddProjectMetadata_ExistingRows(t *testing.T) {
	pool := pgxtest.Connect(t)(t)
	ctx := context.Background()

	files, err := fs.Glob(flowdb.MigrationsFS, "migrations/*.sql")
	require.NoError(t, err)
	sort.Strings(files)
	idx := -1
	for i, f := range files {
		if strings.HasSuffix(f, "_add_project_metadata.sql") {
			idx = i
		}
	}
	require.NotEqual(t, -1, idx)

	apply := func(f string) {
		b, err := fs.ReadFile(flowdb.MigrationsFS, f)
		require.NoError(t, err)
		_, err = pool.Exec(ctx, string(b))
		require.NoError(t, err, f)
	}
	for _, f := range files[:idx] {
		apply(f)
	}

	wid := accountsid.NewWorkspaceID().String()
	pid1, pid2 := id.NewProjectID(), id.NewProjectID()
	for _, pid := range []id.ProjectID{pid1, pid2} {
		_, err := pool.Exec(ctx,
			`INSERT INTO projects (id, workspace_id, name, updated_at) VALUES ($1, $2, 'old', now())`,
			pid.String(), wid)
		require.NoError(t, err)
	}

	for _, f := range files[idx:] {
		apply(f)
	}

	r := postgres.NewProject(pgxx.NewClient(pool))
	for _, pid := range []id.ProjectID{pid1, pid2} {
		got, err := r.FindByID(ctx, pid)
		require.NoError(t, err)
		assert.Equal(t, project.DefaultAlias(pid), got.Alias())
		assert.Equal(t, project.VisibilityPrivate, got.Visibility(), "existing projects stay private")
		assert.Empty(t, got.Topics())
		assert.Zero(t, got.StarCount())
	}
}
