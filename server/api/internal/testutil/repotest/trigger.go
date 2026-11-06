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
