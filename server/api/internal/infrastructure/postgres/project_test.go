package postgres_test

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newProject(wid accountsid.WorkspaceID, pid id.ProjectID, name string) *project.Project {
	return project.New().
		ID(pid).Workspace(wid).
		Name(name).Description("desc").
		UpdatedAt(time.Now()).
		MustBuild()
}

func TestProject_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	p := project.New().
		ID(pid).Workspace(wid).
		Name("My Project").Description("desc").
		IsBasicAuthActive(true).BasicAuthUsername("user").BasicAuthPassword("pass").
		UpdatedAt(time.Now()).
		MustBuild()
	r := postgres.NewProject(pgxx.NewClient(pool))
	require.NoError(t, r.Save(ctx, p))
	got, err := r.FindByID(ctx, pid)
	require.NoError(t, err)
	assert.Equal(t, pid, got.ID())
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, "My Project", got.Name())
	assert.True(t, got.IsBasicAuthActive())
	assert.Equal(t, "user", got.BasicAuthUsername())
}

func TestProject_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	_, err := postgres.NewProject(pgxx.NewClient(pool)).FindByID(context.Background(), id.NewProjectID())
	assert.Error(t, err)
}

func TestProject_FindByIDs_Order(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid1 := id.NewProjectID()
	pid2 := id.NewProjectID()
	require.NoError(t, r.Save(ctx, newProject(wid, pid1, "p1")))
	require.NoError(t, r.Save(ctx, newProject(wid, pid2, "p2")))
	missing := id.NewProjectID()
	got, err := r.FindByIDs(ctx, id.ProjectIDList{pid2, missing, pid1})
	require.NoError(t, err)
	require.Len(t, got, 3)
	assert.Equal(t, pid2, got[0].ID())
	assert.Nil(t, got[1])
	assert.Equal(t, pid1, got[2].ID())
}

func TestProject_FindByWorkspace_ExcludesArchived(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	active := newProject(wid, id.NewProjectID(), "active")
	archived := project.New().ID(id.NewProjectID()).Workspace(wid).
		Name("archived").IsArchived(true).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, active))
	require.NoError(t, r.Save(ctx, archived))
	got, _, err := r.FindByWorkspace(ctx, wid, nil, nil, nil)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, "active", got[0].Name())
}

func TestProject_FindByWorkspace_IncludesArchived(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	require.NoError(t, r.Save(ctx, newProject(wid, id.NewProjectID(), "active")))
	archived := project.New().ID(id.NewProjectID()).Workspace(wid).
		Name("archived").IsArchived(true).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, archived))
	inc := true
	got, _, err := r.FindByWorkspace(ctx, wid, nil, nil, &inc)
	require.NoError(t, err)
	assert.Len(t, got, 2)
}

func TestProject_FindByWorkspace_Paginated(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	for i := 0; i < 5; i++ {
		require.NoError(t, r.Save(ctx, newProject(wid, id.NewProjectID(), "p")))
	}
	page := &interfaces.PaginationParam{Page: &interfaces.PageBasedPaginationParam{Page: 1, PageSize: 2}}
	got, info, err := r.FindByWorkspace(ctx, wid, page, nil, nil)
	require.NoError(t, err)
	assert.Len(t, got, 2)
	assert.Equal(t, int64(5), info.TotalCount)
	assert.Equal(t, 3, info.TotalPages)
}

func TestProject_FindByWorkspace_Keyword(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	require.NoError(t, r.Save(ctx, newProject(wid, id.NewProjectID(), "findme project")))
	require.NoError(t, r.Save(ctx, newProject(wid, id.NewProjectID(), "other")))
	kw := "findme"
	got, _, err := r.FindByWorkspace(ctx, wid, nil, &kw, nil)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, "findme project", got[0].Name())
}

func TestProject_FindByWorkspace_NotReadable(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool)).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	got, info, err := r.FindByWorkspace(ctx, accountsid.NewWorkspaceID(), nil, nil, nil)
	require.NoError(t, err)
	assert.Empty(t, got)
	assert.NotNil(t, info)
}

func TestProject_CountByWorkspace(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	require.NoError(t, r.Save(ctx, newProject(wid, id.NewProjectID(), "p1")))
	require.NoError(t, r.Save(ctx, newProject(wid, id.NewProjectID(), "p2")))
	n, err := r.CountByWorkspace(ctx, wid)
	require.NoError(t, err)
	assert.Equal(t, 2, n)
}

func TestProject_CountByWorkspace_NotReadable(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	r := postgres.NewProject(pgxx.NewClient(pool)).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	_, err := r.CountByWorkspace(ctx, wid)
	assert.ErrorIs(t, err, repo.ErrOperationDenied)
}

func TestProject_CountPublicByWorkspace(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	require.NoError(t, r.Save(ctx, newProject(wid, id.NewProjectID(), "p1")))
	n, err := r.CountPublicByWorkspace(ctx, wid)
	require.NoError(t, err)
	// Flow has no publishment-status concept, so a saved project is never
	// "public" — mirrors Mongo's effective 0 for flow-owned data.
	assert.Equal(t, 0, n)
}

func TestProject_CountPublicByWorkspace_NotReadable(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	r := postgres.NewProject(pgxx.NewClient(pool)).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	_, err := r.CountPublicByWorkspace(ctx, wid)
	assert.ErrorIs(t, err, repo.ErrOperationDenied)
}

func TestProject_Remove(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	require.NoError(t, r.Save(ctx, newProject(wid, pid, "to delete")))
	require.NoError(t, r.Remove(ctx, pid))
	_, err := r.FindByID(ctx, pid)
	assert.Error(t, err)
}

func TestProject_Remove_WithWorkspaceFilter(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	base := postgres.NewProject(pgxx.NewClient(pool))
	wid1 := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	pid1 := id.NewProjectID()
	pid2 := id.NewProjectID()
	require.NoError(t, base.Save(ctx, newProject(wid1, pid1, "p1")))
	require.NoError(t, base.Save(ctx, newProject(wid2, pid2, "p2")))
	r := base.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid1},
		Writable: accountsid.WorkspaceIDList{wid1},
	})
	require.NoError(t, r.Remove(ctx, pid1))
	require.NoError(t, r.Remove(ctx, pid2))
	got, err := base.FindByID(ctx, pid2)
	require.NoError(t, err)
	assert.NotNil(t, got)
}

func TestProject_Save_SharedToken(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	tok := "my-shared-token"
	p := project.New().
		ID(pid).Workspace(wid).Name("tok").
		SharedToken(&tok).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, p))
	got, err := r.FindByID(ctx, pid)
	require.NoError(t, err)
	require.NotNil(t, got.SharedToken())
	assert.Equal(t, "my-shared-token", *got.SharedToken())
}

func TestProject_Save_Upsert(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewProject(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	p := newProject(wid, pid, "original")
	require.NoError(t, r.Save(ctx, p))
	p.SetUpdateName("updated")
	require.NoError(t, r.Save(ctx, p))
	got, err := r.FindByID(ctx, pid)
	require.NoError(t, err)
	assert.Equal(t, "updated", got.Name())
}
