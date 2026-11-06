package gcpscheduler

import (
	"context"
	"testing"
	"time"

	schedulerpb "cloud.google.com/go/scheduler/apiv1/schedulerpb"
	"github.com/googleapis/gax-go/v2"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// fakeClient keeps one job's state the way Cloud Scheduler does: CreateJob and
// UpdateJob leave it as it was, and only PauseJob and ResumeJob change it.
type fakeClient struct {
	SchedulerClient
	paused  []string
	resumed []string
	state   schedulerpb.Job_State
}

func (c *fakeClient) CreateJob(_ context.Context, req *schedulerpb.CreateJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	c.state = schedulerpb.Job_ENABLED
	return &schedulerpb.Job{Name: req.Job.Name, State: c.state}, nil
}

func (c *fakeClient) UpdateJob(_ context.Context, req *schedulerpb.UpdateJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	return &schedulerpb.Job{Name: req.Job.Name, State: c.state}, nil
}

func (c *fakeClient) PauseJob(_ context.Context, req *schedulerpb.PauseJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	c.paused = append(c.paused, req.Name)
	c.state = schedulerpb.Job_PAUSED
	return &schedulerpb.Job{Name: req.Name, State: c.state}, nil
}

func (c *fakeClient) ResumeJob(_ context.Context, req *schedulerpb.ResumeJobRequest, _ ...gax.CallOption) (*schedulerpb.Job, error) {
	c.resumed = append(c.resumed, req.Name)
	c.state = schedulerpb.Job_ENABLED
	return &schedulerpb.Job{Name: req.Name, State: c.state}, nil
}

func newTimeDriven(t *testing.T, enabled bool) *trigger.Trigger {
	t.Helper()
	tr, err := trigger.New().NewID().Workspace(accountsid.NewWorkspaceID()).Deployment(id.NewDeploymentID()).
		Description("t").EventSource(trigger.EventSourceTypeTimeDriven).TimeInterval(trigger.TimeIntervalEveryDay).
		Enabled(enabled).CreatedAt(time.Now()).UpdatedAt(time.Now()).Build()
	require.NoError(t, err)
	return tr
}

func TestSchedulerRepo_JobStateFollowsTheTrigger(t *testing.T) {
	ctx := context.Background()
	newRepo := func() (*SchedulerRepo, *fakeClient) {
		c := &fakeClient{}
		return &SchedulerRepo{client: c, config: SchedulerConfig{ProjectID: "p", Location: "l", Host: "https://flow"}}, c
	}

	t.Run("an enabled trigger's job is created running", func(t *testing.T) {
		s, c := newRepo()

		require.NoError(t, s.CreateScheduledJob(ctx, newTimeDriven(t, true)))

		assert.Equal(t, schedulerpb.Job_ENABLED, c.state)
		assert.Empty(t, c.paused)
	})

	t.Run("a disabled trigger's job is created paused", func(t *testing.T) {
		s, c := newRepo()
		tr := newTimeDriven(t, false)

		require.NoError(t, s.CreateScheduledJob(ctx, tr))

		assert.Equal(t, schedulerpb.Job_PAUSED, c.state)
		assert.Equal(t, []string{"projects/p/locations/l/jobs/" + formatSchedulerJobID(tr.ID().String())}, c.paused)
	})

	t.Run("disabling a trigger pauses its job and enabling resumes it", func(t *testing.T) {
		s, c := newRepo()
		tr := newTimeDriven(t, true)
		require.NoError(t, s.CreateScheduledJob(ctx, tr))

		tr.SetEnabled(false)
		require.NoError(t, s.UpdateScheduledJob(ctx, tr))
		assert.Equal(t, schedulerpb.Job_PAUSED, c.state)

		tr.SetEnabled(true)
		require.NoError(t, s.UpdateScheduledJob(ctx, tr))
		assert.Equal(t, schedulerpb.Job_ENABLED, c.state)
		assert.Len(t, c.paused, 1)
		assert.Len(t, c.resumed, 1)
	})

	t.Run("an edit that keeps the state changes nothing", func(t *testing.T) {
		s, c := newRepo()
		tr := newTimeDriven(t, true)
		require.NoError(t, s.CreateScheduledJob(ctx, tr))

		require.NoError(t, s.UpdateScheduledJob(ctx, tr))

		assert.Empty(t, c.paused)
		assert.Empty(t, c.resumed)
	})

	t.Run("a job can be paused by its trigger", func(t *testing.T) {
		s, c := newRepo()
		tr := newTimeDriven(t, true)
		require.NoError(t, s.CreateScheduledJob(ctx, tr))

		require.NoError(t, s.PauseScheduledJob(ctx, tr.ID()))

		assert.Equal(t, schedulerpb.Job_PAUSED, c.state)
	})
}
