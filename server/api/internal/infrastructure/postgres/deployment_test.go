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
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newDeployment(wid accountsid.WorkspaceID, did id.DeploymentID) *deployment.Deployment {
	return deployment.New().
		ID(did).
		Workspace(wid).
		WorkflowURL("gs://bucket/workflow.yaml").
		Description("desc").
		Version("v1").
		UpdatedAt(time.Now()).
		IsHead(false).
		MustBuild()
}

func TestDeployment_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	d := deployment.New().
		ID(did).Workspace(wid).
		WorkflowURL("gs://bucket/wf.yaml").
		Description("my deployment").
		Version("v1").
		UpdatedAt(time.Now()).
		IsHead(true).
		MustBuild()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	require.NoError(t, r.Save(ctx, d))
	got, err := r.FindByID(ctx, did)
	require.NoError(t, err)
	assert.Equal(t, did, got.ID())
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, "my deployment", got.Description())
	assert.Equal(t, "v1", got.Version())
	assert.True(t, got.IsHead())
}

func TestDeployment_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	_, err := postgres.NewDeployment(pgxx.NewClient(pool)).FindByID(context.Background(), id.NewDeploymentID())
	assert.Error(t, err)
}

func TestDeployment_FindByIDs_Order(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	did1 := id.NewDeploymentID()
	did2 := id.NewDeploymentID()
	require.NoError(t, r.Save(ctx, newDeployment(wid, did1)))
	require.NoError(t, r.Save(ctx, newDeployment(wid, did2)))
	missing := id.NewDeploymentID()
	got, err := r.FindByIDs(ctx, id.DeploymentIDList{did2, missing, did1})
	require.NoError(t, err)
	require.Len(t, got, 2) // missing id omitted (OrderByIDs drops absent ids)
	assert.Equal(t, did2, got[0].ID())
	assert.Equal(t, did1, got[1].ID())
}

func TestDeployment_FindByWorkspace_NoPagination(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	require.NoError(t, r.Save(ctx, newDeployment(wid, id.NewDeploymentID())))
	require.NoError(t, r.Save(ctx, newDeployment(wid, id.NewDeploymentID())))
	require.NoError(t, r.Save(ctx, newDeployment(wid2, id.NewDeploymentID())))
	got, info, err := r.FindByWorkspace(ctx, wid, nil, nil)
	require.NoError(t, err)
	require.NotNil(t, info)
	assert.Len(t, got, 2)
}

func TestDeployment_FindByWorkspace_Paginated(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	for i := 0; i < 5; i++ {
		require.NoError(t, r.Save(ctx, newDeployment(wid, id.NewDeploymentID())))
	}
	page := &interfaces.PaginationParam{Page: &interfaces.PageBasedPaginationParam{Page: 1, PageSize: 2}}
	got, info, err := r.FindByWorkspace(ctx, wid, page, nil)
	require.NoError(t, err)
	assert.Len(t, got, 2)
	assert.Equal(t, int64(5), info.TotalCount)
	assert.Equal(t, 3, info.TotalPages)
}

func TestDeployment_FindByWorkspace_Keyword(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	hay := deployment.New().ID(id.NewDeploymentID()).Workspace(wid).
		WorkflowURL("gs://x").Description("findme special").
		Version("v1").UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, hay))
	require.NoError(t, r.Save(ctx, newDeployment(wid, id.NewDeploymentID())))
	kw := "findme"
	got, _, err := r.FindByWorkspace(ctx, wid, nil, &kw)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, "findme special", got[0].Description())
}

func TestDeployment_FindByWorkspace_NotReadable(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool)).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	got, info, err := r.FindByWorkspace(ctx, accountsid.NewWorkspaceID(), nil, nil)
	require.NoError(t, err)
	assert.Empty(t, got)
	assert.NotNil(t, info)
}

func TestDeployment_FindByProject(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	pidPtr := &pid
	notHead := deployment.New().ID(id.NewDeploymentID()).Workspace(wid).
		WorkflowURL("gs://x").Description("not head").Version("v1").
		UpdatedAt(time.Now()).Project(pidPtr).IsHead(false).MustBuild()
	require.NoError(t, r.Save(ctx, notHead))
	head := deployment.New().ID(id.NewDeploymentID()).Workspace(wid).
		WorkflowURL("gs://x").Description("head dep").Version("v2").
		UpdatedAt(time.Now()).Project(pidPtr).IsHead(true).MustBuild()
	require.NoError(t, r.Save(ctx, head))
	got, err := r.FindByProject(ctx, pid)
	require.NoError(t, err)
	assert.True(t, got.IsHead())
	assert.Equal(t, "head dep", got.Description())
}

func TestDeployment_FindByVersion(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	pidPtr := &pid
	d := deployment.New().ID(id.NewDeploymentID()).Workspace(wid).
		WorkflowURL("gs://x").Description("versioned").Version("v3").
		UpdatedAt(time.Now()).Project(pidPtr).MustBuild()
	require.NoError(t, r.Save(ctx, d))
	got, err := r.FindByVersion(ctx, wid, pidPtr, "v3")
	require.NoError(t, err)
	assert.Equal(t, "v3", got.Version())
	_, err = r.FindByVersion(ctx, wid, pidPtr, "v99")
	assert.Error(t, err)
}

func TestDeployment_FindHead(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	pidPtr := &pid
	head := deployment.New().ID(id.NewDeploymentID()).Workspace(wid).
		WorkflowURL("gs://x").Description("head").Version("v2").
		UpdatedAt(time.Now()).Project(pidPtr).IsHead(true).MustBuild()
	require.NoError(t, r.Save(ctx, head))
	got, err := r.FindHead(ctx, wid, pidPtr)
	require.NoError(t, err)
	assert.True(t, got.IsHead())
	_, err = r.FindHead(ctx, accountsid.NewWorkspaceID(), nil)
	assert.Error(t, err)
}

func TestDeployment_FindVersions(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	pidPtr := &pid
	for _, ver := range []string{"v1", "v2", "v3"} {
		d := deployment.New().ID(id.NewDeploymentID()).Workspace(wid).
			WorkflowURL("gs://x").Description(ver).Version(ver).
			UpdatedAt(time.Now()).Project(pidPtr).MustBuild()
		require.NoError(t, r.Save(ctx, d))
	}
	got, err := r.FindVersions(ctx, wid, pidPtr)
	require.NoError(t, err)
	assert.Len(t, got, 3)
	assert.Equal(t, "v1", got[0].Version())
	assert.Equal(t, "v3", got[2].Version())
}

func TestDeployment_Remove(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	require.NoError(t, r.Save(ctx, newDeployment(wid, did)))
	require.NoError(t, r.Remove(ctx, did))
	_, err := r.FindByID(ctx, did)
	assert.Error(t, err)
}

func TestDeployment_Remove_WithWorkspaceFilter(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	base := postgres.NewDeployment(pgxx.NewClient(pool))
	wid1 := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	did1 := id.NewDeploymentID()
	did2 := id.NewDeploymentID()
	require.NoError(t, base.Save(ctx, newDeployment(wid1, did1)))
	require.NoError(t, base.Save(ctx, newDeployment(wid2, did2)))
	r := base.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid1},
		Writable: accountsid.WorkspaceIDList{wid1},
	})
	require.NoError(t, r.Remove(ctx, did1))
	require.NoError(t, r.Remove(ctx, did2))
	got, err := base.FindByID(ctx, did2)
	require.NoError(t, err)
	assert.NotNil(t, got)
}

func TestDeployment_Save_WithProjectAndHeadID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewDeployment(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	prevHead := id.NewDeploymentID()
	did := id.NewDeploymentID()
	d := deployment.New().
		ID(did).Workspace(wid).
		WorkflowURL("gs://x").Description("with project").
		Version("v2").UpdatedAt(time.Now()).
		Project(&pid).HeadID(&prevHead).IsHead(true).
		MustBuild()
	require.NoError(t, r.Save(ctx, d))
	got, err := r.FindByID(ctx, did)
	require.NoError(t, err)
	require.NotNil(t, got.Project())
	assert.Equal(t, pid, *got.Project())
	require.NotNil(t, got.HeadID())
	assert.Equal(t, prevHead, *got.HeadID())
}
