package postgres_test

import (
	"context"
	"errors"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/pgxx"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newTrig(wid accountsid.WorkspaceID, did id.DeploymentID, tid id.TriggerID) *trigger.Trigger {
	return trigger.New().ID(tid).Workspace(wid).Deployment(did).
		Description("d").EventSource(trigger.EventSourceTypeTimeDriven).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
}

func TestTrigger_Save_FindByID(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	tid := id.NewTriggerID()
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tr := trigger.New().ID(tid).Workspace(wid).Deployment(did).
		Description("desc").EventSource(trigger.EventSourceTypeTimeDriven).
		TimeInterval(trigger.TimeIntervalEveryDay).Enabled(true).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
	r := postgres.NewTrigger(pool)
	require.NoError(t, r.Save(ctx, tr))
	got, err := r.FindByID(ctx, tid)
	require.NoError(t, err)
	assert.Equal(t, tid, got.ID())
	assert.Equal(t, wid, got.Workspace())
	assert.Equal(t, did, got.Deployment())
	assert.Equal(t, "desc", got.Description())
	assert.Equal(t, trigger.EventSourceTypeTimeDriven, got.EventSource())
	assert.True(t, got.Enabled())
}

func TestTrigger_FindByID_NotFound(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	got, err := postgres.NewTrigger(pool).FindByID(context.Background(), id.NewTriggerID())
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_FindByIDs_Order(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid1 := id.NewTriggerID()
	tid2 := id.NewTriggerID()
	for _, x := range []id.TriggerID{tid1, tid2} {
		require.NoError(t, r.Save(ctx, newTrig(wid, did, x)))
	}
	missing := id.NewTriggerID()
	got, err := r.FindByIDs(ctx, id.TriggerIDList{tid2, missing, tid1})
	require.NoError(t, err)
	require.Len(t, got, 3)
	assert.Equal(t, tid2, got[0].ID())
	assert.Nil(t, got[1])
	assert.Equal(t, tid1, got[2].ID())
}

func TestTrigger_FindByDeployment(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	other := id.NewDeploymentID()
	require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))
	require.NoError(t, r.Save(ctx, newTrig(wid, other, id.NewTriggerID())))
	got, err := r.FindByDeployment(ctx, did)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, did, got[0].Deployment())
}

func TestTrigger_Remove(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid := id.NewTriggerID()
	require.NoError(t, r.Save(ctx, newTrig(wid, did, tid)))
	require.NoError(t, r.Remove(ctx, tid))
	got, err := r.FindByID(ctx, tid)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Remove_WithWorkspaceFilter(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	base := postgres.NewTrigger(pool)
	wid1 := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid1 := id.NewTriggerID()
	tid2 := id.NewTriggerID()
	require.NoError(t, base.Save(ctx, newTrig(wid1, did, tid1)))
	require.NoError(t, base.Save(ctx, newTrig(wid2, did, tid2)))
	r := base.Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{wid1},
		Writable: accountsid.WorkspaceIDList{wid1},
	})
	require.NoError(t, r.Remove(ctx, tid1))
	require.NoError(t, r.Remove(ctx, tid2)) // not writable -> no-op
	got, err := base.FindByID(ctx, tid2)
	require.NoError(t, err)
	assert.NotNil(t, got)
}

func TestTrigger_FindByWorkspace_NoPagination(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	wid := accountsid.NewWorkspaceID()
	wid2 := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))
	require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))
	require.NoError(t, r.Save(ctx, newTrig(wid2, did, id.NewTriggerID())))
	got, info, err := r.FindByWorkspace(ctx, wid, nil, nil)
	require.NoError(t, err)
	require.NotNil(t, info)
	assert.Len(t, got, 2)
}

func TestTrigger_FindByWorkspace_Paginated(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	for i := 0; i < 5; i++ {
		require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))
	}
	page := &interfaces.PaginationParam{Page: &interfaces.PageBasedPaginationParam{Page: 1, PageSize: 2}}
	got, info, err := r.FindByWorkspace(ctx, wid, page, nil)
	require.NoError(t, err)
	assert.Len(t, got, 2)
	assert.Equal(t, int64(5), info.TotalCount)
	assert.Equal(t, 3, info.TotalPages)
	assert.Equal(t, 1, info.CurrentPage)
}

func TestTrigger_FindByWorkspace_Keyword(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	hay := trigger.New().ID(id.NewTriggerID()).Workspace(wid).Deployment(did).
		Description("findme please").EventSource(trigger.EventSourceTypeTimeDriven).
		CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, r.Save(ctx, hay))
	require.NoError(t, r.Save(ctx, newTrig(wid, did, id.NewTriggerID())))
	kw := "findme"
	got, _, err := r.FindByWorkspace(ctx, wid, nil, &kw)
	require.NoError(t, err)
	assert.Len(t, got, 1)
	assert.Equal(t, "findme please", got[0].Description())
}

func TestTrigger_FindByWorkspace_NotReadable(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	wid := accountsid.NewWorkspaceID()
	r := postgres.NewTrigger(pool).Filtered(repo.WorkspaceFilter{
		Readable: accountsid.WorkspaceIDList{accountsid.NewWorkspaceID()},
	})
	got, info, err := r.FindByWorkspace(ctx, wid, nil, nil)
	require.NoError(t, err)
	assert.Empty(t, got)
	assert.NotNil(t, info)
}

func TestTrigger_Save_RollsBackInTransaction(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid := id.NewTriggerID()
	// Return a non-nil error to trigger rollback.
	_ = pgxx.NewTransactor(pool, 0).WithinTransaction(ctx, func(ctx context.Context) error {
		require.NoError(t, r.Save(ctx, newTrig(wid, did, tid)))
		return errors.New("rollback")
	})
	got, err := r.FindByID(ctx, tid)
	assert.Error(t, err)
	assert.Nil(t, got)
}

func TestTrigger_Save_CommitsInTransaction(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()
	r := postgres.NewTrigger(pool)
	wid := accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()
	tid := id.NewTriggerID()
	require.NoError(t, pgxx.NewTransactor(pool, 0).WithinTransaction(ctx, func(ctx context.Context) error {
		return r.Save(ctx, newTrig(wid, did, tid))
	}))
	got, err := r.FindByID(ctx, tid)
	require.NoError(t, err)
	assert.Equal(t, tid, got.ID())
}
