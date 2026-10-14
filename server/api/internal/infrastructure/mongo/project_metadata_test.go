package mongo

import (
	"context"
	"os"
	"sync"
	"testing"
	"time"

	"github.com/google/uuid"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

// connectProjectDB opts these tests into REEARTH_FLOW_DB directly. The package's
// other tests go through mongotest, whose Env is never set here, so they skip.
func connectProjectDB(t *testing.T) *Project {
	t.Helper()
	uri := os.Getenv("REEARTH_FLOW_DB")
	if uri == "" {
		t.Skip("REEARTH_FLOW_DB is not set")
	}
	ctx := context.Background()
	c, err := mongo.Connect(ctx, options.Client().ApplyURI(uri).SetConnectTimeout(10*time.Second))
	require.NoError(t, err)
	db := c.Database("test_" + uuid.NewString())
	t.Cleanup(func() {
		_ = db.Drop(context.Background())
		_ = c.Disconnect(context.Background())
	})
	return NewProject(mongox.NewClientWithDatabase(db))
}

func TestProject_SaveFindMetadata(t *testing.T) {
	r := connectProjectDB(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	p := project.New().NewID().Workspace(wid).
		Alias("my-project").Visibility(project.VisibilityPublic).
		Readme("# Hello").License("MIT").Topics([]string{"gis", "plateau"}).
		UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, p))

	got, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.Equal(t, "my-project", got.Alias())
	assert.Equal(t, project.VisibilityPublic, got.Visibility())
	assert.Equal(t, "# Hello", got.Readme())
	assert.Equal(t, "MIT", got.License())
	assert.Equal(t, []string{"gis", "plateau"}, got.Topics())
	assert.Zero(t, got.StarCount())
}

func TestProject_LegacyDocumentGetsDefaults(t *testing.T) {
	r := connectProjectDB(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	// Saved before aliases and visibility existed: the old Save wrote an empty alias.
	_, err := r.client.Client().InsertOne(ctx, bson.M{"id": pid.String(), "workspace": wid.String(), "alias": "", "name": "old"})
	require.NoError(t, err)

	got, err := r.FindByID(ctx, pid)
	require.NoError(t, err)
	assert.Equal(t, project.DefaultAlias(pid), got.Alias())
	assert.Equal(t, project.VisibilityPrivate, got.Visibility())

	byAlias, err := r.FindByAlias(ctx, wid, project.DefaultAlias(pid))
	require.NoError(t, err)
	assert.Equal(t, pid, byAlias.ID())
}

func TestProject_FindByAlias(t *testing.T) {
	r := connectProjectDB(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	other := accountsid.NewWorkspaceID()
	p := project.New().NewID().Workspace(wid).Alias("my-project").MustBuild()
	require.NoError(t, r.Save(ctx, p))

	got, err := r.FindByAlias(ctx, wid, "my-project")
	require.NoError(t, err)
	assert.Equal(t, p.ID(), got.ID())

	_, err = r.FindByAlias(ctx, other, "my-project")
	assert.ErrorIs(t, err, rerror.ErrNotFound, "aliases are per workspace")

	_, err = r.FindByAlias(ctx, wid, "missing")
	assert.ErrorIs(t, err, rerror.ErrNotFound)

	filtered := r.Filtered(repo.WorkspaceFilter{Readable: accountsid.WorkspaceIDList{other}})
	_, err = filtered.FindByAlias(ctx, wid, "my-project")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
}

func TestProject_SetStar(t *testing.T) {
	r := connectProjectDB(t)
	ctx := context.Background()
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).MustBuild()
	require.NoError(t, r.Save(ctx, p))
	u1, u2 := accountsid.NewUserID(), accountsid.NewUserID()

	require.NoError(t, r.SetStar(ctx, p.ID(), u1, true))
	require.NoError(t, r.SetStar(ctx, p.ID(), u1, true))
	require.NoError(t, r.SetStar(ctx, p.ID(), u2, true))
	got, err := r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.Equal(t, int64(2), got.StarCount(), "starring twice counts once")

	require.NoError(t, r.SetStar(ctx, p.ID(), u1, false))
	got, err = r.FindByID(ctx, p.ID())
	require.NoError(t, err)
	assert.False(t, got.IsStarredBy(u1))
	assert.True(t, got.IsStarredBy(u2))

	assert.ErrorIs(t, r.SetStar(ctx, id.NewProjectID(), u1, true), rerror.ErrNotFound)
}

func TestProject_SetStar_Concurrent(t *testing.T) {
	r := connectProjectDB(t)
	ctx := context.Background()
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).MustBuild()
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
	r := connectProjectDB(t)
	ctx := context.Background()
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).Name("before").MustBuild()
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

func TestProject_AliasUniqueIndex(t *testing.T) {
	r := connectProjectDB(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()

	// Documents saved before aliases existed all hold an empty alias; the index
	// must still build over them.
	for range 2 {
		_, err := r.client.Client().InsertOne(ctx, bson.M{"id": id.NewProjectID().String(), "workspace": wid.String(), "alias": ""})
		require.NoError(t, err)
	}
	require.NoError(t, r.Init(ctx))
	require.NoError(t, r.Init(ctx), "a restart finds the index already there")

	require.NoError(t, r.Save(ctx, project.New().NewID().Workspace(wid).Alias("taken").MustBuild()))
	err := r.Save(ctx, project.New().NewID().Workspace(wid).Alias("taken").MustBuild())
	assert.ErrorIs(t, err, interfaces.ErrProjectAliasAlreadyUsed)

	assert.NoError(t, r.Save(ctx, project.New().NewID().Workspace(accountsid.NewWorkspaceID()).Alias("taken").MustBuild()),
		"aliases are per workspace")
}

func TestProject_AliasUniqueIndex_Concurrent(t *testing.T) {
	r := connectProjectDB(t)
	ctx := context.Background()
	require.NoError(t, r.Init(ctx))
	wid := accountsid.NewWorkspaceID()

	const n = 10
	errs := make(chan error, n)
	var wg sync.WaitGroup
	for range n {
		wg.Add(1)
		go func() {
			defer wg.Done()
			errs <- r.Save(ctx, project.New().NewID().Workspace(wid).Alias("contested").MustBuild())
		}()
	}
	wg.Wait()
	close(errs)

	saved := 0
	for err := range errs {
		if err == nil {
			saved++
			continue
		}
		assert.ErrorIs(t, err, interfaces.ErrProjectAliasAlreadyUsed)
	}
	assert.Equal(t, 1, saved)
}
