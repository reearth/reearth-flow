package interactor

import (
	"context"
	"errors"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// fakeScheduler records what is asked of the scheduler.
type fakeScheduler struct {
	listErr error
	jobs    []gateway.ScheduledTriggerJob
	updated []id.TriggerID
	deleted []id.TriggerID
}

func (s *fakeScheduler) CreateScheduledJob(context.Context, *trigger.Trigger) error { return nil }

func (s *fakeScheduler) UpdateScheduledJob(_ context.Context, t *trigger.Trigger) error {
	s.updated = append(s.updated, t.ID())
	return nil
}

func (s *fakeScheduler) DeleteScheduledJob(_ context.Context, triggerID id.TriggerID) error {
	s.deleted = append(s.deleted, triggerID)
	return nil
}

func (s *fakeScheduler) GetScheduledJob(context.Context, id.TriggerID) (*gateway.ScheduledJobInfo, error) {
	return nil, nil
}

func (s *fakeScheduler) ListScheduledJobs(context.Context) ([]gateway.ScheduledTriggerJob, error) {
	return s.jobs, s.listErr
}

func (s *fakeScheduler) Close() error { return nil }

func saveTrigger(t *testing.T, r *memory.Trigger, source trigger.EventSourceType) *trigger.Trigger {
	t.Helper()
	b := trigger.New().NewID().Workspace(accountsid.NewWorkspaceID()).Deployment(id.NewDeploymentID()).
		Description("t").Enabled(true).EventSource(source).CreatedAt(time.Now()).UpdatedAt(time.Now())
	if source == "TIME_DRIVEN" {
		b = b.TimeInterval(trigger.TimeIntervalEveryDay)
	}
	tr := b.MustBuild()
	require.NoError(t, r.Save(context.Background(), tr))
	return tr
}

func TestReconcileScheduledJobs(t *testing.T) {
	t.Parallel()
	ctx := context.Background()

	t.Run("deletes jobs left behind, adds the token where missing, leaves the rest", func(t *testing.T) {
		t.Parallel()
		repo := memory.NewTrigger()
		upToDate := saveTrigger(t, repo, "TIME_DRIVEN")
		noToken := saveTrigger(t, repo, "TIME_DRIVEN")
		nowAPI := saveTrigger(t, repo, "API_DRIVEN")
		gone := id.NewTriggerID()
		s := &fakeScheduler{jobs: []gateway.ScheduledTriggerJob{
			{TriggerID: upToDate.ID(), Authenticated: true},
			{TriggerID: noToken.ID(), Authenticated: false},
			{TriggerID: nowAPI.ID(), Authenticated: false},
			{TriggerID: gone, Authenticated: false},
		}}

		require.NoError(t, ReconcileScheduledJobs(ctx, repo, s))

		assert.Equal(t, []id.TriggerID{noToken.ID()}, s.updated)
		assert.Equal(t, []id.TriggerID{gone}, s.deleted, "an existing trigger's job is never deleted, even if it is not time-driven")
	})

	t.Run("one job whose trigger is gone is deleted", func(t *testing.T) {
		t.Parallel()
		gone := id.NewTriggerID()
		s := &fakeScheduler{jobs: []gateway.ScheduledTriggerJob{{TriggerID: gone, Authenticated: true}}}

		require.NoError(t, ReconcileScheduledJobs(ctx, memory.NewTrigger(), s))

		assert.Equal(t, []id.TriggerID{gone}, s.deleted)
	})

	t.Run("deletes nothing when no job has a trigger", func(t *testing.T) {
		t.Parallel()
		s := &fakeScheduler{jobs: []gateway.ScheduledTriggerJob{
			{TriggerID: id.NewTriggerID()},
			{TriggerID: id.NewTriggerID()},
		}}

		assert.Error(t, ReconcileScheduledJobs(ctx, memory.NewTrigger(), s))
		assert.Empty(t, s.deleted)
		assert.Empty(t, s.updated)
	})

	t.Run("a scheduler that cannot list changes nothing", func(t *testing.T) {
		t.Parallel()
		s := &fakeScheduler{listErr: errors.New("permission denied")}

		assert.Error(t, ReconcileScheduledJobs(ctx, memory.NewTrigger(), s))
		assert.Empty(t, s.deleted)
	})

	t.Run("no scheduler, nothing to do", func(t *testing.T) {
		t.Parallel()
		assert.NoError(t, ReconcileScheduledJobs(ctx, memory.NewTrigger(), nil))
	})
}
