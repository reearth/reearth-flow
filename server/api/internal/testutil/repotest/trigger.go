package repotest

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TriggerCountEnabled checks repo.Trigger.CountEnabledByWorkspace: only the
// workspace's enabled triggers count, API-driven and time-driven alike.
func TriggerCountEnabled(t *testing.T, r repo.Trigger) {
	t.Helper()
	ctx := context.Background()
	ws, other := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()
	did := id.NewDeploymentID()

	save := func(ws accountsid.WorkspaceID, source trigger.EventSourceType, enabled bool) *trigger.Trigger {
		t.Helper()
		b := trigger.New().NewID().Workspace(ws).Deployment(did).Description("t").
			EventSource(source).Enabled(enabled).CreatedAt(time.Now()).UpdatedAt(time.Now())
		if source == trigger.EventSourceTypeTimeDriven {
			b = b.TimeInterval(trigger.TimeIntervalEveryDay)
		}
		tr := b.MustBuild()
		require.NoError(t, r.Save(ctx, tr))
		return tr
	}
	count := func(ws accountsid.WorkspaceID) int {
		t.Helper()
		n, err := r.CountEnabledByWorkspace(ctx, ws)
		require.NoError(t, err)
		return n
	}

	assert.Zero(t, count(ws), "no triggers yet")

	save(ws, trigger.EventSourceTypeTimeDriven, true)
	save(ws, trigger.EventSourceTypeAPIDriven, true)
	disabled := save(ws, trigger.EventSourceTypeTimeDriven, false)
	save(other, trigger.EventSourceTypeAPIDriven, true)

	assert.Equal(t, 2, count(ws), "enabled triggers of both kinds, in this workspace only")

	disabled.SetEnabled(true)
	require.NoError(t, r.Save(ctx, disabled))
	assert.Equal(t, 3, count(ws), "enabling a trigger counts it")
}

// TriggerSaveLastRefusal checks repo.Trigger.SaveLastRefusal: it changes only
// the refusal, leaves a trigger that no longer exists, or that the repository
// cannot write, as it is, and clears the refusal when given none.
func TriggerSaveLastRefusal(t *testing.T, newRepo func(*testing.T) repo.Trigger) {
	t.Helper()
	ctx := context.Background()
	at := time.Date(2026, 11, 6, 0, 3, 0, 0, time.UTC)
	until := at.Add(2 * time.Hour)
	refusal := &trigger.Refusal{At: at, Code: trigger.RefusalTimeDrivenTriggerRunsDailyLimit, RetryUntil: &until}
	newTrigger := func(ws accountsid.WorkspaceID, description string) *trigger.Trigger {
		return trigger.New().NewID().Workspace(ws).Deployment(id.NewDeploymentID()).
			Description(description).EventSource(trigger.EventSourceTypeTimeDriven).
			TimeInterval(trigger.TimeIntervalEveryDay).Enabled(true).
			CreatedAt(at).UpdatedAt(at).MustBuild()
	}

	t.Run("changes only the refusal", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)

		tr := newTrigger(accountsid.NewWorkspaceID(), "nightly")
		require.NoError(t, r.Save(ctx, tr))
		require.NoError(t, r.SaveLastRefusal(ctx, tr.ID(), refusal))

		got, err := r.FindByID(ctx, tr.ID())
		require.NoError(t, err)
		assert.Equal(t, "nightly", got.Description())
		assert.True(t, got.Enabled())
		require.NotNil(t, got.LastRefusal())
		assert.True(t, at.Equal(got.LastRefusal().At))
		assert.Equal(t, trigger.RefusalTimeDrivenTriggerRunsDailyLimit, got.LastRefusal().Code)
		require.NotNil(t, got.LastRefusal().RetryUntil)
		assert.True(t, until.Equal(*got.LastRefusal().RetryUntil))

		require.NoError(t, r.SaveLastRefusal(ctx, tr.ID(), nil))
		got, err = r.FindByID(ctx, tr.ID())
		require.NoError(t, err)
		assert.Nil(t, got.LastRefusal())
	})

	t.Run("does not create a trigger that no longer exists", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)

		tr := newTrigger(accountsid.NewWorkspaceID(), "removed")
		require.NoError(t, r.Save(ctx, tr))
		require.NoError(t, r.Remove(ctx, tr.ID()))
		require.NoError(t, r.SaveLastRefusal(ctx, tr.ID(), refusal))

		_, err := r.FindByID(ctx, tr.ID())
		assert.Error(t, err)
	})

	t.Run("leaves a trigger it cannot write", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)

		writable, other := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()
		tr := newTrigger(other, "elsewhere")
		require.NoError(t, r.Save(ctx, tr))
		filtered := r.Filtered(repo.WorkspaceFilter{
			Readable: accountsid.WorkspaceIDList{writable, other},
			Writable: accountsid.WorkspaceIDList{writable},
		})
		require.NoError(t, filtered.SaveLastRefusal(ctx, tr.ID(), refusal))

		got, err := r.FindByID(ctx, tr.ID())
		require.NoError(t, err)
		assert.Nil(t, got.LastRefusal())
	})
}
