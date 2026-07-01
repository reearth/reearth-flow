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
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newJob(wid accountsid.WorkspaceID, jid id.JobID) *job.Job {
	return job.New().
		ID(jid).Workspace(wid).
		GCPJobID("gcp-123").Status(job.StatusPending).
		StartedAt(time.Now()).
		MustBuild()
}

func newDebugJob(wid accountsid.WorkspaceID, jid id.JobID, pid id.ProjectID) *job.Job {
	t := true
	return job.New().
		ID(jid).Workspace(wid).
		GCPJobID("gcp-debug").Status(job.StatusRunning).
		StartedAt(time.Now()).
		Debug(&t).
		ProjectID(&pid).
		MustBuild()
}

func TestJob_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	jid := id.NewJobID()
	j := job.New().
		ID(jid).Workspace(wid).
		GCPJobID("gcp-abc").Status(job.StatusRunning).
		LogsURL("https://logs.example.com").
		StartedAt(time.Now()).
		MustBuild()
	r := postgres.NewJob(pgxx.NewClient(pool))
	require.NoError(t, r.Save(ctx, j))
	got, err := r.FindByID(ctx, jid)
	require.NoError(t, err)
	assert.Equal(t, jid, got.ID())
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, "gcp-abc", got.GCPJobID())
	assert.Equal(t, "https://logs.example.com", got.LogsURL())
}

func TestJob_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	_, err := postgres.NewJob(pgxx.NewClient(pool)).FindByID(context.Background(), id.NewJobID())
	assert.Error(t, err)
}

func TestJob_FindByIDs_Order(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	jid1 := id.NewJobID()
	jid2 := id.NewJobID()
	require.NoError(t, r.Save(ctx, newJob(wid, jid1)))
	require.NoError(t, r.Save(ctx, newJob(wid, jid2)))
	missing := id.NewJobID()
	got, err := r.FindByIDs(ctx, id.JobIDList{jid2, missing, jid1})
	require.NoError(t, err)
	require.Len(t, got, 2) // missing id omitted (OrderByIDs drops absent ids)
	assert.Equal(t, jid2, got[0].ID())
	assert.Equal(t, jid1, got[1].ID())
}

func TestJob_FindByWorkspace_ExcludesDebug(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	require.NoError(t, r.Save(ctx, newJob(wid, id.NewJobID())))
	require.NoError(t, r.Save(ctx, newDebugJob(wid, id.NewJobID(), pid)))
	got, _, err := r.FindByWorkspace(ctx, wid, nil, nil)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	for _, j := range got {
		assert.Nil(t, j.Debug())
	}
}

func TestJob_FindByWorkspace_Paginated(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	for i := 0; i < 5; i++ {
		require.NoError(t, r.Save(ctx, newJob(wid, id.NewJobID())))
	}
	page := &interfaces.PaginationParam{Page: &interfaces.PageBasedPaginationParam{Page: 1, PageSize: 2}}
	got, info, err := r.FindByWorkspace(ctx, wid, page, nil)
	require.NoError(t, err)
	assert.Len(t, got, 2)
	assert.Equal(t, int64(5), info.TotalCount)
	assert.Equal(t, 3, info.TotalPages)
}

func TestJob_FindByWorkspace_Keyword(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	running := job.New().ID(id.NewJobID()).Workspace(wid).
		GCPJobID("x").Status(job.StatusRunning).
		StartedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, running))
	require.NoError(t, r.Save(ctx, newJob(wid, id.NewJobID())))
	kw := "RUNNING"
	got, _, err := r.FindByWorkspace(ctx, wid, nil, &kw)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, job.StatusRunning, got[0].Status())
}

func TestJob_FindByWorkspace_NotReadable(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool)).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	got, info, err := r.FindByWorkspace(ctx, accountsid.NewWorkspaceID(), nil, nil)
	require.NoError(t, err)
	assert.Empty(t, got)
	assert.NotNil(t, info)
}

func TestJob_FindByProject_ReturnsDebugJobsOnly(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	require.NoError(t, r.Save(ctx, newJob(wid, id.NewJobID())))
	require.NoError(t, r.Save(ctx, newDebugJob(wid, id.NewJobID(), pid)))
	require.NoError(t, r.Save(ctx, newDebugJob(wid, id.NewJobID(), pid)))
	got, err := r.FindByProject(ctx, pid)
	require.NoError(t, err)
	assert.Len(t, got, 2)
	for _, j := range got {
		require.NotNil(t, j.Debug())
		assert.True(t, *j.Debug())
	}
}

func TestJob_Mode_RoundTripAndPreviewSchemaExcluded(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	debug := true

	runJob := newDebugJob(wid, id.NewJobID(), pid) // mode defaults to run
	previewID := id.NewJobID()
	previewJob := job.New().
		ID(previewID).Workspace(wid).ProjectID(&pid).
		Debug(&debug).Mode(job.ModePreviewSchema).
		GCPJobID("gcp-x").Status(job.StatusPending).StartedAt(time.Now()).
		MustBuild()
	require.NoError(t, r.Save(ctx, runJob))
	require.NoError(t, r.Save(ctx, previewJob))

	// Mode round-trips through Postgres.
	got, err := r.FindByID(ctx, previewID)
	require.NoError(t, err)
	assert.Equal(t, job.ModePreviewSchema, got.Mode())

	// FindByProject excludes the preview-schema job (run history must stay clean).
	byProj, err := r.FindByProject(ctx, pid)
	require.NoError(t, err)
	require.Len(t, byProj, 1)
	assert.Equal(t, runJob.ID(), byProj[0].ID())
	assert.Equal(t, job.ModeRun, byProj[0].Mode())
}

func TestJob_RemoveByProject(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	pid := id.NewProjectID()
	jid1 := id.NewJobID()
	jid2 := id.NewJobID()
	require.NoError(t, r.Save(ctx, newDebugJob(wid, jid1, pid)))
	require.NoError(t, r.Save(ctx, newDebugJob(wid, jid2, pid)))
	require.NoError(t, r.RemoveByProject(ctx, pid))
	got, err := r.FindByProject(ctx, pid)
	require.NoError(t, err)
	assert.Empty(t, got)
}

func TestJob_Remove(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	jid := id.NewJobID()
	require.NoError(t, r.Save(ctx, newJob(wid, jid)))
	require.NoError(t, r.Remove(ctx, jid))
	_, err := r.FindByID(ctx, jid)
	assert.Error(t, err)
}

func TestJob_Remove_WithWorkspaceFilter(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	base := postgres.NewJob(pgxx.NewClient(pool))
	wid1 := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	jid1 := id.NewJobID()
	jid2 := id.NewJobID()
	require.NoError(t, base.Save(ctx, newJob(wid1, jid1)))
	require.NoError(t, base.Save(ctx, newJob(wid2, jid2)))
	r := base.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid1},
		Writable: accountsid.WorkspaceIDList{wid1},
	})
	require.NoError(t, r.Remove(ctx, jid1))
	require.NoError(t, r.Remove(ctx, jid2))
	got, err := base.FindByID(ctx, jid2)
	require.NoError(t, err)
	assert.NotNil(t, got)
}

func TestJob_Save_Upsert(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewJob(pgxx.NewClient(pool))
	wid := accountsid.NewWorkspaceID()
	jid := id.NewJobID()
	j := newJob(wid, jid)
	require.NoError(t, r.Save(ctx, j))
	j.SetStatus(job.StatusCompleted)
	require.NoError(t, r.Save(ctx, j))
	got, err := r.FindByID(ctx, jid)
	require.NoError(t, err)
	assert.Equal(t, job.StatusCompleted, got.Status())
}
