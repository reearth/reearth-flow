package interactor

import (
	"context"
	"strings"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// The fixture already has a job running, so a call whose token is accepted is
// then refused with ErrJobAlreadyRunning.
func assertTokenAccepted(t *testing.T, f *planFixture, i interfaces.Trigger, tid id.TriggerID, token string) {
	t.Helper()
	_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tid, AuthenticationToken: token})
	assert.ErrorIs(t, err, interfaces.ErrJobAlreadyRunning)
}

func assertTokenRefused(t *testing.T, f *planFixture, i interfaces.Trigger, tid id.TriggerID, token string) {
	t.Helper()
	_, err := i.ExecuteAPITrigger(f.ctx, interfaces.ExecuteAPITriggerParam{TriggerID: tid, AuthenticationToken: token})
	assert.ErrorContains(t, err, "invalid auth token")
}

func createAPITrigger(t *testing.T, f *planFixture, i interfaces.Trigger) (*trigger.Trigger, string) {
	t.Helper()
	tr, token, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
		WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Description: "t", Enabled: true, EventSource: "API_DRIVEN",
	})
	require.NoError(t, err)
	return tr, token
}

func TestTrigger_AuthToken(t *testing.T) {
	t.Run("an API trigger is created with a generated token", func(t *testing.T) {
		f := newPlanFixture(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		tr, token := createAPITrigger(t, f, i)

		assert.True(t, strings.HasPrefix(token, trigger.AuthTokenPrefix), token)
		assertTokenAccepted(t, f, i, tr.ID(), token)
	})

	t.Run("a scheduled trigger is created without a token", func(t *testing.T) {
		f := newPlanFixture(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		tr, token, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: f.deployment(t).ID(), Description: "t", Enabled: true,
			EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryDay,
		})

		require.NoError(t, err)
		assert.Empty(t, token)
		assert.Nil(t, tr.AuthTokenHash())
	})

	t.Run("re-generating a token stops the old one working", func(t *testing.T) {
		f := newPlanFixture(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)
		tr, old := createAPITrigger(t, f, i)

		_, token, err := i.RegenerateAuthToken(f.ctx, tr.ID())

		require.NoError(t, err)
		assert.NotEqual(t, old, token)
		assertTokenRefused(t, f, i, tr.ID(), old)
		assertTokenAccepted(t, f, i, tr.ID(), token)
	})

	t.Run("only an API trigger's token can be re-generated", func(t *testing.T) {
		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, _, err := i.RegenerateAuthToken(f.ctx, tr.ID())

		assert.ErrorIs(t, err, interfaces.ErrTriggerNotAPIDriven)
	})

	t.Run("re-generating a token needs permission to edit the trigger", func(t *testing.T) {
		f := newPlanFixture(t)
		tr, old := createAPITrigger(t, f, NewTrigger(f.repo, f.gw, f.job, f.perm))
		noEdit := NewMockPermissionChecker(func(_ context.Context, _, action string) (bool, error) {
			return action != rbac.ActionEdit, nil
		})
		i := NewTrigger(f.repo, f.gw, f.job, noEdit)

		_, _, err := i.RegenerateAuthToken(f.ctx, tr.ID())

		assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
		assertTokenAccepted(t, f, i, tr.ID(), old)
	})

	t.Run("updating an API trigger keeps its token", func(t *testing.T) {
		f := newPlanFixture(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)
		tr, old := createAPITrigger(t, f, i)
		desc := "renamed"

		_, token, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{ID: tr.ID(), Description: &desc, EventSource: "API_DRIVEN"})

		require.NoError(t, err)
		assert.Empty(t, token, "no new token is issued")
		assertTokenAccepted(t, f, i, tr.ID(), old)
	})

	t.Run("a trigger switched to API-driven is given a token", func(t *testing.T) {
		f := newPlanFixture(t)
		tr := f.trigger(t, trigger.New().EventSource("TIME_DRIVEN").TimeInterval(trigger.TimeIntervalEveryDay))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, token, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{ID: tr.ID(), EventSource: "API_DRIVEN"})

		require.NoError(t, err)
		assert.True(t, strings.HasPrefix(token, trigger.AuthTokenPrefix), token)
		assertTokenAccepted(t, f, i, tr.ID(), token)
	})

	t.Run("a trigger switched to a schedule loses its token", func(t *testing.T) {
		f := newPlanFixture(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)
		tr, _ := createAPITrigger(t, f, i)

		_, token, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{
			ID: tr.ID(), EventSource: "TIME_DRIVEN", TimeInterval: trigger.TimeIntervalEveryDay,
		})

		require.NoError(t, err)
		assert.Empty(t, token)
		stored, err := f.repo.Trigger.FindByID(f.ctx, tr.ID())
		require.NoError(t, err)
		assert.Nil(t, stored.AuthTokenHash())
		assert.Nil(t, stored.AuthTokenHint())
	})
}
