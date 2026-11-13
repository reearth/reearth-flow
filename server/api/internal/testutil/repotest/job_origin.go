package repotest

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// JobTriggerAndOrigin checks that a job keeps its trigger and origin.
func JobTriggerAndOrigin(t *testing.T, newRepo func(*testing.T) repo.Job) {
	t.Helper()
	ctx := context.Background()

	t.Run("keeps the trigger and the CMS item", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		tid := id.NewTriggerID()
		origin := &job.Origin{CMSItem: &job.CMSItemOrigin{
			ProjectID: "p1", ModelID: "m1", ItemID: "i1", ConnectionID: "c1",
			ProjectName: "Wayfarer", ModelName: "Buildings", ItemTitle: "Shinjuku 3-chome",
		}}
		j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Status(job.StatusPending).
			StartedAt(time.Now()).Trigger(&tid).Origin(origin).MustBuild()
		require.NoError(t, r.Save(ctx, j))

		got, err := r.FindByID(ctx, j.ID())

		require.NoError(t, err)
		require.NotNil(t, got.Trigger())
		assert.Equal(t, tid, *got.Trigger())
		assert.Equal(t, origin, got.Origin())
		assert.Nil(t, got.ScheduledRun(), "a trigger alone is not a scheduled run")
	})

	t.Run("a job started in Flow has neither", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Status(job.StatusPending).
			StartedAt(time.Now()).MustBuild()
		require.NoError(t, r.Save(ctx, j))

		got, err := r.FindByID(ctx, j.ID())

		require.NoError(t, err)
		assert.Nil(t, got.Trigger())
		assert.Nil(t, got.Origin())
	})

	t.Run("two runs of one trigger without a scheduled time do not conflict", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		ws := accountsid.NewWorkspaceID()
		tid := id.NewTriggerID()
		for range 2 {
			j := job.New().NewID().Workspace(ws).Status(job.StatusPending).StartedAt(time.Now()).Trigger(&tid).MustBuild()
			require.NoError(t, r.Save(ctx, j))
		}
	})
}
